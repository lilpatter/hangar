// Valve KeyValues: the text format (loginusers.vdf, localconfig.vdf, *.acf) and the binary appinfo.vdf cache.
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Node {
    Str(String),
    Obj(Vec<(String, Node)>),
}

impl Node {
    /// Case-insensitive lookup, the way Steam itself reads keys.
    pub fn get(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Obj(items) => items.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v),
            Node::Str(_) => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Node> {
        match self {
            Node::Obj(items) => items.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v),
            Node::Str(_) => None,
        }
    }

    pub fn path(&self, keys: &[&str]) -> Option<&Node> {
        keys.iter().try_fold(self, |n, k| n.get(k))
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Node::Str(s) => Some(s),
            Node::Obj(_) => None,
        }
    }

    pub fn num(&self, key: &str) -> Option<u64> {
        self.str(key)?.trim().parse().ok()
    }

    pub fn entries(&self) -> &[(String, Node)] {
        match self {
            Node::Obj(items) => items,
            Node::Str(_) => &[],
        }
    }

    pub fn entries_mut(&mut self) -> Option<&mut Vec<(String, Node)>> {
        match self {
            Node::Obj(items) => Some(items),
            Node::Str(_) => None,
        }
    }

    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Sets a string value, keeping the key's original spelling and position if it exists.
    pub fn set(&mut self, key: &str, value: &str) {
        if let Node::Obj(items) = self {
            match items.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(key)) {
                Some((_, v)) => *v = Node::Str(value.into()),
                None => items.push((key.into(), Node::Str(value.into()))),
            }
        }
    }
}

// ---------- Text format ----------

pub fn parse(text: &str) -> Result<Node, String> {
    let mut p = Parser { s: text.trim_start_matches('\u{feff}').as_bytes(), i: 0 };
    let items = p.block(true)?;
    Ok(Node::Obj(items))
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn skip(&mut self) {
        while self.i < self.s.len() {
            match self.s[self.i] {
                b' ' | b'\t' | b'\r' | b'\n' => self.i += 1,
                b'/' if self.s.get(self.i + 1) == Some(&b'/') => {
                    while self.i < self.s.len() && self.s[self.i] != b'\n' {
                        self.i += 1;
                    }
                }
                // Platform conditionals like [$WIN32] are ignored.
                b'[' => {
                    while self.i < self.s.len() && self.s[self.i] != b']' {
                        self.i += 1;
                    }
                    self.i += 1;
                }
                _ => break,
            }
        }
    }

    fn token(&mut self) -> Result<String, String> {
        if self.s.get(self.i) == Some(&b'"') {
            self.i += 1;
            let mut out = Vec::new();
            while self.i < self.s.len() {
                let c = self.s[self.i];
                self.i += 1;
                match c {
                    b'"' => return String::from_utf8(out).map_err(|e| e.to_string()),
                    b'\\' if self.i < self.s.len() => {
                        let e = self.s[self.i];
                        self.i += 1;
                        out.push(match e {
                            b'n' => b'\n',
                            b't' => b'\t',
                            other => other,
                        });
                    }
                    _ => out.push(c),
                }
            }
            Err("Unterminated string in VDF".into())
        } else {
            let start = self.i;
            while self.i < self.s.len() && !matches!(self.s[self.i], b' ' | b'\t' | b'\r' | b'\n' | b'{' | b'}' | b'"') {
                self.i += 1;
            }
            Ok(String::from_utf8_lossy(&self.s[start..self.i]).into_owned())
        }
    }

    fn block(&mut self, top: bool) -> Result<Vec<(String, Node)>, String> {
        let mut items = Vec::new();
        loop {
            self.skip();
            match self.s.get(self.i) {
                None if top => return Ok(items),
                None => return Err("Unexpected end of VDF".into()),
                Some(b'}') => {
                    self.i += 1;
                    return Ok(items);
                }
                _ => {}
            }
            let key = self.token()?;
            self.skip();
            if self.s.get(self.i) == Some(&b'{') {
                self.i += 1;
                items.push((key, Node::Obj(self.block(false)?)));
            } else {
                items.push((key, Node::Str(self.token()?)));
            }
        }
    }
}

pub fn write(node: &Node) -> String {
    fn esc(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }
    fn go(items: &[(String, Node)], depth: usize, out: &mut String) {
        let tabs = "\t".repeat(depth);
        for (k, v) in items {
            match v {
                Node::Str(s) => out.push_str(&format!("{tabs}\"{}\"\t\t\"{}\"\n", esc(k), esc(s))),
                Node::Obj(children) => {
                    out.push_str(&format!("{tabs}\"{}\"\n{tabs}{{\n", esc(k)));
                    go(children, depth + 1, out);
                    out.push_str(&format!("{tabs}}}\n"));
                }
            }
        }
    }
    let mut out = String::new();
    go(node.entries(), 0, &mut out);
    out
}

// ---------- Binary appinfo.vdf (v28/v29) ----------

/// Each app's "common" section, keyed by appid.
pub fn parse_appinfo(data: &[u8]) -> Result<HashMap<u32, Node>, String> {
    let mut r = Bin { b: data, i: 0, strings: Vec::new() };
    let magic = r.u32()?;
    let _universe = r.u32()?;
    let version = magic & 0xff;
    if magic >> 8 != 0x075644 || !(0x27..=0x29).contains(&version) {
        return Err(format!("Unsupported appinfo.vdf format {magic:#x}"));
    }
    if version >= 0x29 {
        let table = r.u64()? as usize;
        let mut t = Bin { b: data, i: table, strings: Vec::new() };
        let count = t.u32()?;
        for _ in 0..count {
            let s = t.cstr()?;
            r.strings.push(s);
        }
    }
    let header = if version >= 0x28 { 60 } else { 40 };
    let mut apps = HashMap::new();
    loop {
        let appid = r.u32()?;
        if appid == 0 {
            break;
        }
        let size = r.u32()? as usize;
        let end = r.i + size;
        if end > data.len() {
            return Err("Truncated appinfo.vdf".into());
        }
        r.i += header;
        let node = Node::Obj(r.kv(end)?);
        if let Some(common) = node.path(&["appinfo", "common"]) {
            apps.insert(appid, common.clone());
        }
        r.i = end;
    }
    Ok(apps)
}

struct Bin<'a> {
    b: &'a [u8],
    i: usize,
    strings: Vec<String>,
}

impl Bin<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let s = self.b.get(self.i..self.i + n).ok_or("Truncated appinfo.vdf")?;
        self.i += n;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn cstr(&mut self) -> Result<String, String> {
        let rest = &self.b[self.i.min(self.b.len())..];
        let n = rest.iter().position(|&c| c == 0).ok_or("Unterminated string in appinfo.vdf")?;
        let s = String::from_utf8_lossy(&rest[..n]).into_owned();
        self.i += n + 1;
        Ok(s)
    }
    fn key(&mut self) -> Result<String, String> {
        if self.strings.is_empty() {
            self.cstr()
        } else {
            let idx = self.u32()? as usize;
            self.strings.get(idx).cloned().ok_or_else(|| "Bad string index in appinfo.vdf".into())
        }
    }
    fn kv(&mut self, end: usize) -> Result<Vec<(String, Node)>, String> {
        let mut items = Vec::new();
        while self.i < end {
            let t = self.take(1)?[0];
            if t == 0x08 {
                return Ok(items);
            }
            let k = self.key()?;
            let v = match t {
                0x00 => Node::Obj(self.kv(end)?),
                0x01 => Node::Str(self.cstr()?),
                0x02 | 0x04 | 0x06 => Node::Str((self.u32()? as i32).to_string()),
                0x03 => Node::Str(f32::from_le_bytes(self.take(4)?.try_into().unwrap()).to_string()),
                0x07 => Node::Str(self.u64()?.to_string()),
                0x0a => Node::Str((self.u64()? as i64).to_string()),
                other => return Err(format!("Unknown value type {other:#x} in appinfo.vdf")),
            };
            items.push((k, v));
        }
        Ok(items)
    }
}

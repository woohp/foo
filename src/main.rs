use std::collections::BTreeMap;
use std::str::{from_utf8, from_utf8_unchecked};
use std::result::Result;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::net::Ipv4Addr;

mod err;
use err::BencodeError;
mod kademlia;
use kademlia::{NodeId, Node};


#[derive(Debug)]
enum BencodeObject {
    Integer(i64),
    Bytes(Vec<u8>),
    List(Vec<BencodeObject>),
    Dict(BTreeMap<String, BencodeObject>)
}

impl BencodeObject {
    fn parse<S: Into<Vec<u8>>>(_bytes: S) -> Result<BencodeObject, BencodeError> {
        let bytes = _bytes.into();
        let mut i = 0;
        let len = bytes.len();
        let bencode_object = _parse(&bytes, &mut i)?;
        if i == len {
            Ok(bencode_object)
        } else {
            Err(BencodeError::UnexpectedCharacter(i))
        }
    }

    fn into_bytes(&self) -> Vec<u8> {
        match *self {
            BencodeObject::Integer(ref i) => format!("i{}e", i).into_bytes(),
            BencodeObject::Bytes(ref bytes) => {
                let mut final_bytes = format!("{}:", bytes.len()).into_bytes();
                final_bytes.extend(bytes);
                final_bytes
            },
            BencodeObject::List(ref list) => {
                let mut final_bytes = vec![b'l'];
                for o in list {
                    final_bytes.extend(o.into_bytes());
                }
                final_bytes.push(b'e');
                final_bytes
            },
            BencodeObject::Dict(ref dict) => {
                let mut final_bytes = vec![b'd'];
                for (key, value) in dict {
                    final_bytes.extend(key.as_bytes());
                    final_bytes.extend(value.into_bytes());
                }
                final_bytes.push(b'e');
                final_bytes
            },
        }
    }
}

trait Bencodeable {
    fn bencode(self) -> BencodeObject;
}

impl Bencodeable for BencodeObject {
    fn bencode(self) -> BencodeObject {
        self
    }
}

impl Bencodeable for i64 {
    fn bencode(self) -> BencodeObject {
        BencodeObject::Integer(self)
    }
}

impl Bencodeable for Vec<BencodeObject> {
    fn bencode(self) -> BencodeObject {
        BencodeObject::List(self)
    }
}

impl Bencodeable for Vec<u8> {
    fn bencode(self) -> BencodeObject {
        BencodeObject::Bytes(self)
    }
}

impl Bencodeable for BTreeMap<String, BencodeObject> {
    fn bencode(self) -> BencodeObject {
        BencodeObject::Dict(self)
    }
}

impl Bencodeable for String {
    fn bencode(self) -> BencodeObject {
        BencodeObject::Bytes(self.as_bytes().to_vec())
    }
}

impl Bencodeable for &'static str {
    fn bencode(self) -> BencodeObject {
        BencodeObject::Bytes(self.as_bytes().to_vec())
    }
}


macro_rules! bencode (
    { $($key:expr => $value:expr),+ } => {{
        let mut map = BTreeMap::new();
        $(
            map.insert($key.to_string(), $value.bencode());
        )+
        BencodeObject::Dict(map)
    }};
    { $($x:expr),* } => {{
        let mut vec = Vec::new();
        $(
            vec.push($x.bencode());
        )*
        BencodeObject::List(vec)
    }};
);

fn _parse(bytes: &[u8], i: &mut usize) -> Result<BencodeObject, BencodeError> {
    if *i == bytes.len() {
        return Err(BencodeError::UnexpectedEndOfInput)
    }

    match bytes[*i] {
        b'i' => {
            *i += 1;
            let start = *i;
            while *i < bytes.len() && ((bytes[*i] >= b'0' && bytes[*i] <= b'9') || bytes[*i] == b'-') {
                *i += 1;
            }
            if *i == bytes.len() {
                return Err(BencodeError::UnexpectedEndOfInput);
            }
            if bytes[*i] != b'e' {
                return Err(BencodeError::UnexpectedCharacter(*i));
            }
            *i += 1;
            let n = unsafe { from_utf8_unchecked(&bytes[start .. *i-1]) }.parse::<i64>()?;
            return Ok(BencodeObject::Integer(n));
        },
        b'l' => {
            *i += 1;
            let mut vec = Vec::new();
            while *i < bytes.len() && bytes[*i] != b'e' {
                vec.push(_parse(&bytes, i)?);
            }
            if *i == bytes.len() {
                return Err(BencodeError::UnexpectedEndOfInput);
            }
            *i += 1;

            return Ok(BencodeObject::List(vec));
        },
        b'd' => {
            *i += 1;
            let mut map = BTreeMap::new();
            while *i < bytes.len() && bytes[*i] != b'e' {
                let key = match _parse(&bytes, i)? {
                    BencodeObject::Bytes(bytes) => from_utf8(&bytes)?.to_string(),
                    _ => return Err(BencodeError::DictionaryKeyNotString)
                };
                let value = _parse(&bytes, i)?;
                map.insert(key, value);
            }
            if *i == bytes.len() {
                return Err(BencodeError::UnexpectedEndOfInput);
            }
            *i += 1;

            return Ok(BencodeObject::Dict(map));
        },
        b'0' ... b'9' => {
            let start = *i;
            while *i < bytes.len() && (bytes[*i] >= b'0' && bytes[*i] <= b'9') {
                *i += 1;
            }
            if *i == bytes.len() {
                return Err(BencodeError::UnexpectedEndOfInput);
            }
            if bytes[*i] != b':' {
                return Err(BencodeError::UnexpectedCharacter(*i));
            }
            let n = unsafe { from_utf8_unchecked(&bytes[start .. *i]) }.parse::<usize>()?;
            *i += 1;
            let bytes = &bytes[*i .. *i+n];
            *i += n;

            return Ok(BencodeObject::Bytes(bytes.to_vec()));
        },
        _ => Err(BencodeError::UnexpectedCharacter(*i))
    }
}


fn file_to_bytes(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    File::open(path).and_then(|mut file| {
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}


fn main() {
    println!("{:?}", bencode![1 => 1, 2 => 2, 3 => bencode![1, "2", 3]]);
    println!("{:?}", bencode![1, "2", 3]);
    println!("{:?}", from_utf8(&bencode![1, "2", 3].into_bytes()));
    println!("{:?}", BencodeObject::parse("4:asdf"));
    println!("{:?}", BencodeObject::parse("4:asdf"));
    println!("{:?}", BencodeObject::parse("l4:asdf3:asde"));
    println!("{:?}", BencodeObject::parse("d1:a1:a2:bb2:bbe"));
    println!("{:?}", BencodeObject::parse("i-12345fe"));
    println!("{:?}", BencodeObject::parse("li-12345e4:asdfe"));

    let path = Path::new("/Users/huipeng/Downloads/ubuntu-16.10-desktop-amd64.iso.torrent");
    match file_to_bytes(path)
        .map(|file_bytes| BencodeObject::parse(file_bytes)) {

        Ok(obj) => println!("{:?}", obj),
        e => println!("{:?}", e)
    };

    let node = Node {
        id: NodeId {data: [1, 2, 3, 4, 5]},
        ip_address: Ipv4Addr::new(127, 0, 0, 1),
        port: 1234
    };
    println!("{:?}", node);
}

use rusftp_macro::TaggedEnum;

#[derive(Debug, Clone, TaggedEnum, Default)]
#[repr(u8)]
pub enum Foo {
    A(u8) = 15,
    B(String) = 127,
    C {
        v: u8,
    } = 128,
    #[default]
    D = 200,
}

fn test_one(foo: Foo) {
    use serde::{Deserialize, Serialize};
    eprintln!("Input: {foo:?}");
    eprintln!("Kind: {:?}", foo.kind());

    let mut ser = super::message::encoder::SftpEncoder { buf: Vec::new() };
    if let Err(err) = foo.serialize(&mut ser) {
        eprintln!("Serialization error: {err:?}");
        return;
    }
    eprintln!("Serialized: {:?}", ser.buf);
    let mut de = super::message::decoder::SftpDecoder {
        buf: ser.buf.as_slice(),
    };
    match Foo::deserialize(&mut de) {
        Ok(foo) => eprintln!("Deserialized: {foo:?}"),
        Err(err) => eprintln!("Deserialization error: {err:?}"),
    }
}

pub fn test() {
    test_one(Foo::A(3));
    test_one(Foo::B("hello".into()));
    test_one(Foo::C { v: 56 });
    test_one(Foo::D);
}

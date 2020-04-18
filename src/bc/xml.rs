// YaSerde currently macro-expands names like __type__value from type_
***REMOVED***![allow(non_snake_case)]

use std::io::{Read, Write};
// YaSerde is currently naming the traits and the derive macros identically
use yaserde_derive::{YaDeserialize, YaSerialize};
use yaserde::{ser::Config, YaDeserialize, YaSerialize};
// YaSerde currently needs this imported
***REMOVED***[allow(pub_use_of_private_extern_crate)]
use yaserde::log;

***REMOVED***[cfg(test)]
use indoc::indoc;

***REMOVED***[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
***REMOVED***[yaserde(rename="body")]
pub struct Body {
    ***REMOVED***[yaserde(rename="Encryption")]
    pub encryption: Option<Encryption>,
    ***REMOVED***[yaserde(rename="LoginUser")]
    pub login_user: Option<LoginUser>,
    ***REMOVED***[yaserde(rename="LoginNet")]
    pub login_net: Option<LoginNet>,
}

impl Body {
    pub fn try_parse(s: impl Read) -> Result<Self, String> {
        yaserde::de::from_reader(s)
    }
    pub fn serialize<W: Write>(&self, w: W) -> Result<W, String> {
        yaserde::ser::serialize_with_writer(self, w, &Config::default())
    }
}

***REMOVED***[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct Encryption {
    ***REMOVED***[yaserde(attribute)]
    pub version: String,
    ***REMOVED***[yaserde(rename="type")]
    pub type_: String,
    pub nonce: String,
}

***REMOVED***[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct LoginUser {
    ***REMOVED***[yaserde(attribute)]
    pub version: String,
    ***REMOVED***[yaserde(rename="userName")]
    pub user_name: String,
    pub password: String,
    ***REMOVED***[yaserde(rename="userVer")]
    pub user_ver: u32,
}

***REMOVED***[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct LoginNet {
    ***REMOVED***[yaserde(attribute)]
    pub version: String,
    ***REMOVED***[yaserde(rename="type")]
    pub type_: String,
    ***REMOVED***[yaserde(rename="udpPort")]
    pub udp_port: u16,
}

***REMOVED***[test]
fn test_encryption_deser() {
    let sample = indoc!(r***REMOVED***"
        <?xml version="1.0" encoding="UTF-8" ?>
        <body>
        <Encryption version="1.1">
        <type>md5</type>
        <nonce>9E6D1FCB9E69846D</nonce>
        </Encryption>
        </body>"***REMOVED***);
    let b: Body = yaserde::de::from_str(sample).unwrap();
    let enc = b.encryption.unwrap();

    assert_eq!(enc.version, "1.1");
    assert_eq!(enc.nonce, "9E6D1FCB9E69846D");
    assert_eq!(enc.type_, "md5");
}

***REMOVED***[test]
fn test_login_deser() {
    let sample = indoc!(r***REMOVED***"
        <?xml version="1.0" encoding="UTF-8" ?>
        <body>
        <LoginUser version="1.1">
        <userName>9F07915E819A076E2E14169830769D6</userName>
        <password>8EFECD610524A98390F118D2789BE3B</password>
        <userVer>1</userVer>
        </LoginUser>
        <LoginNet version="1.1">
        <type>LAN</type>
        <udpPort>0</udpPort>
        </LoginNet>
        </body>"***REMOVED***);
    let b: Body = yaserde::de::from_str(sample).unwrap();
    let login_user = b.login_user.unwrap();
    let login_net = b.login_net.unwrap();

    assert_eq!(login_user.version, "1.1");
    assert_eq!(login_user.user_name, "9F07915E819A076E2E14169830769D6");
    assert_eq!(login_user.password, "8EFECD610524A98390F118D2789BE3B");
    assert_eq!(login_user.user_ver, 1);

    assert_eq!(login_net.version, "1.1");
    assert_eq!(login_net.type_, "LAN");
    assert_eq!(login_net.udp_port, 0);
}

***REMOVED***[test]
fn test_login_ser() {
    let sample = indoc!(r***REMOVED***"
        <?xml version="1.0" encoding="UTF-8" ?>
        <body>
        <LoginUser version="1.1">
        <userName>9F07915E819A076E2E14169830769D6</userName>
        <password>8EFECD610524A98390F118D2789BE3B</password>
        <userVer>1</userVer>
        </LoginUser>
        <LoginNet version="1.1">
        <type>LAN</type>
        <udpPort>0</udpPort>
        </LoginNet>
        </body>"***REMOVED***);

    let b = Body {
        login_user: Some(LoginUser {
            version: "1.1".to_string(),
            user_name: "9F07915E819A076E2E14169830769D6".to_string(),
            password: "8EFECD610524A98390F118D2789BE3B".to_string(),
            user_ver: 1,
        }),
        login_net: Some(LoginNet {
            version: "1.1".to_string(),
            type_: "LAN".to_string(),
            udp_port: 0,
        }),
        ..Body::default()
    };

    let b2 = Body::try_parse(sample.as_bytes()).unwrap();
    let b3 = Body::try_parse(yaserde::ser::to_string(&b).unwrap().as_bytes()).unwrap();

    assert_eq!(b, b2);
    assert_eq!(b, b3);
    assert_eq!(b2, b3);
}

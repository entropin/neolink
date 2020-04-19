// YaSerde currently macro-expands names like __type__value from type_
***REMOVED***![allow(non_snake_case)]

use std::io::{Read, Write};
// YaSerde is currently naming the traits and the derive macros identically
use yaserde_derive::{YaDeserialize, YaSerialize};
use yaserde::{ser::Config, YaDeserialize, YaSerialize};

***REMOVED***[cfg(test)]
use indoc::indoc;

***REMOVED***[derive(PartialEq, Eq, Debug, YaDeserialize)]
***REMOVED***[yaserde(flatten)]
pub(super) enum AllTopXmls {
    ***REMOVED***[yaserde(rename="body")]
    BcXml(BcXml),
    Extension(Extension),
}

// Required for YaDeserialize
impl Default for AllTopXmls {
    fn default() -> Self {
        AllTopXmls::BcXml(Default::default())
    }
}

***REMOVED***[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
***REMOVED***[yaserde(root="body")]
pub struct BcXml {
    ***REMOVED***[yaserde(rename="Encryption")]
    pub encryption: Option<Encryption>,
    ***REMOVED***[yaserde(rename="LoginUser")]
    pub login_user: Option<LoginUser>,
    ***REMOVED***[yaserde(rename="LoginNet")]
    pub login_net: Option<LoginNet>,
    ***REMOVED***[yaserde(rename="DeviceInfo")]
    pub device_info: Option<DeviceInfo>,
    ***REMOVED***[yaserde(rename="Preview")]
    pub preview: Option<Preview>,
}

impl AllTopXmls {
    pub fn try_parse(s: impl Read) -> Result<Self, String> {
        yaserde::de::from_reader(s)
    }
}

impl BcXml {
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

***REMOVED***[derive(PartialEq, Eq, Debug, YaDeserialize, YaSerialize)]
pub struct LoginNet {
    ***REMOVED***[yaserde(attribute)]
    pub version: String,
    ***REMOVED***[yaserde(rename="type")]
    pub type_: String,
    ***REMOVED***[yaserde(rename="udpPort")]
    pub udp_port: u16,
}

impl Default for LoginNet {
    fn default() -> Self {
        LoginNet {
            version: xml_ver(),
            type_: "LAN".to_string(),
            udp_port: 0,
        }
    }
}

***REMOVED***[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct DeviceInfo {
    pub resolution: Resolution,
}

***REMOVED***[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct Resolution {
    ***REMOVED***[yaserde(rename="resolutionName")]
    pub name: String,
    pub width: u32,
    pub height: u32,
}

***REMOVED***[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct Preview {
    ***REMOVED***[yaserde(attribute)]
    pub version: String,

    ***REMOVED***[yaserde(rename="channelId")]
    pub channel_id: u32,
    pub handle: u32,
    ***REMOVED***[yaserde(rename="streamType")]
    pub stream_type: String,
}

***REMOVED***[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct Extension {
    ***REMOVED***[yaserde(rename="binaryData")]
    pub binary_data: u32,
}

pub fn xml_ver() -> String {
    "1.1".to_string()
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
    let b: BcXml = yaserde::de::from_str(sample).unwrap();
    let enc = b.encryption.as_ref().unwrap();

    assert_eq!(enc.version, "1.1");
    assert_eq!(enc.nonce, "9E6D1FCB9E69846D");
    assert_eq!(enc.type_, "md5");

    let t = AllTopXmls::try_parse(sample.as_bytes()).unwrap();
    match t {
        AllTopXmls::BcXml(top_b) if top_b == b => assert!(true),
        _ => assert!(false)
    }
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
    let b: BcXml = yaserde::de::from_str(sample).unwrap();
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

    let b = BcXml {
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
        ..BcXml::default()
    };

    let b2 = BcXml::try_parse(sample.as_bytes()).unwrap();
    let b3 = BcXml::try_parse(b.serialize(vec!()).unwrap().as_slice()).unwrap();

    assert_eq!(b, b2);
    assert_eq!(b, b3);
    assert_eq!(b2, b3);
}

***REMOVED***[test]
fn test_deviceinfo_partial_deser() {
    let sample = indoc!(r***REMOVED***"
        <?xml version="1.0" encoding="UTF-8" ?>
        <body>
        <DeviceInfo version="1.1">
        <ipChannel>0</ipChannel>
        <analogChnNum>1</analogChnNum>
        <resolution>
        <resolutionName>3840*2160</resolutionName>
        <width>3840</width>
        <height>2160</height>
        </resolution>
        <language>English</language>
        <sdCard>0</sdCard>
        <ptzMode>none</ptzMode>
        <typeInfo>IPC</typeInfo>
        <softVer>33554880</softVer>
        <B485>0</B485>
        <supportAutoUpdate>0</supportAutoUpdate>
        <userVer>1</userVer>
        </DeviceInfo>
        </body>"***REMOVED***);

    // Needs to ignore all the other crap that we don't care about
    let b = BcXml::try_parse(sample.as_bytes()).unwrap();
    match b {
        BcXml {
            device_info: Some(DeviceInfo {
                resolution: Resolution {
                    width: 3840,
                    height: 2160,
                    ..
                }, ..
            }), ..
        } => assert!(true),
        _ => assert!(false)
    }
}

***REMOVED***[test]
fn test_binary_deser() {
    let _ = env_logger::builder().is_test(true).try_init();

    let sample = indoc!(r***REMOVED***"
        <?xml version="1.0" encoding="UTF-8" ?>
        <Extension version="1.1">
        <binaryData>1</binaryData>
        </Extension>
    "***REMOVED***);
    let b = AllTopXmls::try_parse(sample.as_bytes()).unwrap();
    match b {
        AllTopXmls::Extension(Extension { binary_data: 1 }) => assert!(true),
        _ => assert!(false)
    }
}

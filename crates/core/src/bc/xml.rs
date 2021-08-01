// YaSerde currently macro-expands names like __type__value from type_
#![allow(non_snake_case)]

use std::io::{Read, Write};
// YaSerde is currently naming the traits and the derive macros identically
use yaserde::{ser::Config, YaDeserialize, YaSerialize};
use yaserde_derive::{YaDeserialize, YaSerialize};

#[cfg(test)]
use indoc::indoc;

/// There are two types of payloads xml and binary
#[derive(PartialEq, Eq, Debug, YaDeserialize)]
#[yaserde(flatten)]
pub enum BcPayloads {
    /// XML payloads are the more common ones and include payloads for camera controls
    #[yaserde(rename = "body")]
    BcXml(BcXml),
    /// Binary payloads are received from the camera for streams and sent to the camera
    /// for talk-back and firmware updates
    #[yaserde(flatten)]
    Binary(Vec<u8>),
}

// Required for YaDeserialize
impl Default for BcPayloads {
    fn default() -> Self {
        BcPayloads::Binary(Default::default())
    }
}

/// The top level BC Xml
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
#[yaserde(rename = "body")]
pub struct BcXml {
    /// Encryption xml are negociated during login and contain the NONCE: `MSG_ID_LOGIN`
    #[yaserde(rename = "Encryption")]
    pub encryption: Option<Encryption>,
    /// LoginUser xml is used during modern login: `MSG_ID_LOGIN`
    #[yaserde(rename = "LoginUser")]
    pub login_user: Option<LoginUser>,
    /// LoginNet xml is used during modern login: `MSG_ID_LOGIN`
    #[yaserde(rename = "LoginNet")]
    pub login_net: Option<LoginNet>,
    /// The final part of a login sequence will return DeviceInfo xml: `MSG_ID_LOGIN`
    #[yaserde(rename = "DeviceInfo")]
    pub device_info: Option<DeviceInfo>,
    /// The VersionInfo xml is recieved in reply to a version request: `MSG_ID_VERSION`
    #[yaserde(rename = "VersionInfo")]
    pub version_info: Option<VersionInfo>,
    /// Preview xml is used as part of the stream request to set the stream quality and channel
    #[yaserde(rename = "Preview")]
    pub preview: Option<Preview>,
    #[yaserde(rename = "SystemGeneral")]
    /// SystemGeneral xml is sent or recieved as part of the clock get/setting:
    /// `MSG_ID_GET_GENERAL` and `MSG_ID_SET_GENERAL`
    pub system_general: Option<SystemGeneral>,
    /// Received as part of the Genral system info request: `MSG_ID_GET_GENERAL`
    #[yaserde(rename = "Norm")]
    pub norm: Option<Norm>,
}

impl BcXml {
    pub(crate) fn try_parse(s: impl Read) -> Result<Self, String> {
        yaserde::de::from_reader(s)
    }
    pub(crate) fn serialize<W: Write>(&self, w: W) -> Result<W, String> {
        yaserde::ser::serialize_with_writer(self, w, &Config::default())
    }
}

impl Extension {
    pub(crate) fn try_parse(s: impl Read) -> Result<Self, String> {
        yaserde::de::from_reader(s)
    }
    pub(crate) fn serialize<W: Write>(&self, w: W) -> Result<W, String> {
        yaserde::ser::serialize_with_writer(self, w, &Config::default())
    }
}

/// Encryption xml
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct Encryption {
    /// XML Version always "1.1"
    #[yaserde(attribute)]
    pub version: String,
    #[yaserde(rename = "type")]
    /// The hashing algorithm used. Only observed the value of "md5"
    pub type_: String,
    /// The nonce used to negociate the login and to generate the AES key
    pub nonce: String,
}

/// LoginUser xml
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct LoginUser {
    /// XML Version always "1.1"
    #[yaserde(attribute)]
    pub version: String,
    /// Username to login as
    #[yaserde(rename = "userName")]
    pub user_name: String,
    /// Password for login in plain text
    pub password: String,
    /// Unknown always `1`
    #[yaserde(rename = "userVer")]
    pub user_ver: u32,
}

/// LoginNet xml
#[derive(PartialEq, Eq, Debug, YaDeserialize, YaSerialize)]
pub struct LoginNet {
    /// XML Version always "1.1"
    #[yaserde(attribute)]
    pub version: String,
    /// Type of connection usually LAN (even on wifi)
    #[yaserde(rename = "type")]
    pub type_: String,
    /// The port for the udp will be `0` for tcp
    #[yaserde(rename = "udpPort")]
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

/// DeviceInfo xml
///
/// There is more to this xml but we don't deserialize it all
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct DeviceInfo {
    /// The resolution xml block
    pub resolution: Resolution,
}

/// VersionInfo xml
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct VersionInfo {
    /// Name assigned to the camera
    pub name: String,
    /// Camera's serial number
    pub serialNumber: String,
    /// The camera build day e.g. `"build 19110800"`
    pub buildDay: String,
    /// The hardware version e.g. `"IPC_517SD5"`
    pub hardwareVersion: String,
    /// The config version e.g. `"v2.0.0.0"`
    pub cfgVersion: String,
    /// Firmware version usually a combination of config and build versions e.g.
    /// `"v2.0.0.587_19110800"`
    pub firmwareVersion: String,
    /// Unusure possibly a more detailed hardware version e.g. `"IPC_51716M110000000100000"`
    pub detail: String,
}

/// Resolution xml
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct Resolution {
    /// Resolution name is in the format "width*height" i.e. "2304*1296"
    #[yaserde(rename = "resolutionName")]
    pub name: String,
    /// Height of the stream in pixels
    pub width: u32,
    /// Width of the stream in pixels
    pub height: u32,
}

/// Preview xml
///
/// This xml is used to request a stream to start
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct Preview {
    /// XML Version always "1.1"
    #[yaserde(attribute)]
    pub version: String,

    /// Channel id is usually zero unless using a NVR
    #[yaserde(rename = "channelId")]
    pub channel_id: u8,
    /// Handle usually 0 for mainStream and 1 for subStream
    pub handle: u32,
    /// Either `"mainStream"` or `"subStream"`
    #[yaserde(rename = "streamType")]
    pub stream_type: String,
}

/// Extension xml
///
/// This is used to describe the subsequent payload passed the `payload_offset`
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct Extension {
    /// XML Version always "1.1"
    #[yaserde(attribute)]
    pub version: String,
    /// If the subsequent payload is binary this will be set to 1. Otherwise it is ommited
    #[yaserde(rename = "binaryData")]
    pub binary_data: Option<u32>,
    /// Certain requests such `AbilitySupport` require to know which user this
    /// ability support request is for (why camera dosen't know this based on who
    /// is logged in is unknown... Possible security hole)
    #[yaserde(rename = "userName")]
    pub user_name: Option<String>,
    /// Certain requests such as `AbilitySupport` require details such as what type of
    /// abilities are you intested in. This is a comma seperated list such as
    /// `"system, network, alarm, record, video, image"`
    pub token: Option<String>,
    /// The channel ID. This is usually `0` unless using an NVR
    #[yaserde(rename = "channelId")]
    pub channel_id: Option<u8>,
}

/// SystemGeneral xml
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct SystemGeneral {
    /// XML Version always "1.1"
    #[yaserde(attribute)]
    pub version: String,

    /// Time zone is negative seconds offset from UTC. So +7:00 is -25200
    #[yaserde(rename = "timeZone")]
    pub time_zone: Option<i32>,
    /// Current year
    pub year: Option<i32>,
    /// Current month
    pub month: Option<u8>,
    /// Current day
    pub day: Option<u8>,
    /// Current hour
    pub hour: Option<u8>,
    /// Current minute
    pub minute: Option<u8>,
    /// Current second
    pub second: Option<u8>,

    /// Format to use for On Screen Display usually `"DMY"`
    #[yaserde(rename = "osdFormat")]
    pub osd_format: Option<String>,
    /// Unknown usually `0`
    #[yaserde(rename = "timeFormat")]
    pub time_format: Option<u8>,

    /// Language e.g. `English` will set the language on the reolink app
    pub language: Option<String>,
    /// Name assigned to the camera
    #[yaserde(rename = "deviceName")]
    pub device_name: Option<String>,
}

/// Norm xml
#[derive(PartialEq, Eq, Default, Debug, YaDeserialize, YaSerialize)]
pub struct Norm {
    /// XML Version always "1.1"
    #[yaserde(attribute)]
    pub version: String,
    // This is usually just `"NTSC"`
    norm: String,
}

/// Convience function to return the xml function used throughout the library
pub fn xml_ver() -> String {
    "1.1".to_string()
}

#[test]
fn test_encryption_deser() {
    let sample = indoc!(
        r#"
        <?xml version="1.0" encoding="UTF-8" ?>
        <body>
        <Encryption version="1.1">
        <type>md5</type>
        <nonce>9E6D1FCB9E69846D</nonce>
        </Encryption>
        </body>"#
    );
    let b: BcXml = yaserde::de::from_str(sample).unwrap();
    let enc = b.encryption.as_ref().unwrap();

    assert_eq!(enc.version, "1.1");
    assert_eq!(enc.nonce, "9E6D1FCB9E69846D");
    assert_eq!(enc.type_, "md5");

    let t = BcXml::try_parse(sample.as_bytes()).unwrap();
    match t {
        top_b if top_b == b => {}
        _ => panic!(),
    }
}

#[test]
fn test_login_deser() {
    let sample = indoc!(
        r#"
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
        </body>"#
    );
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

#[test]
fn test_login_ser() {
    let sample = indoc!(
        r#"
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
        </body>"#
    );

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
    let b3 = BcXml::try_parse(b.serialize(vec![]).unwrap().as_slice()).unwrap();

    assert_eq!(b, b2);
    assert_eq!(b, b3);
    assert_eq!(b2, b3);
}

#[test]
fn test_deviceinfo_partial_deser() {
    let sample = indoc!(
        r#"
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
        </body>"#
    );

    // Needs to ignore all the other crap that we don't care about
    let b = BcXml::try_parse(sample.as_bytes()).unwrap();
    match b {
        BcXml {
            device_info:
                Some(DeviceInfo {
                    resolution:
                        Resolution {
                            width: 3840,
                            height: 2160,
                            ..
                        },
                    ..
                }),
            ..
        } => {}
        _ => panic!(),
    }
}

#[test]
fn test_binary_deser() {
    let _ = env_logger::builder().is_test(true).try_init();

    let sample = indoc!(
        r#"
        <?xml version="1.0" encoding="UTF-8" ?>
        <Extension version="1.1">
        <binaryData>1</binaryData>
        </Extension>
    "#
    );
    let b = Extension::try_parse(sample.as_bytes()).unwrap();
    match b {
        Extension {
            binary_data: Some(1),
            ..
        } => {}
        _ => panic!(),
    }
}
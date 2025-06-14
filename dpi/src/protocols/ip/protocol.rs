use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

/// IANA: https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum IpNextLevelProtocol {
    HOPOPT = 0,
    ICMP = 1,
    IGMP = 2,
    GGP = 3,
    IPv4 = 4,
    ST = 5,
    TCP = 6,
    CBT = 7,
    EGP = 8,
    IGP = 9,
    BbnRccMon = 10,
    NvpII = 11,
    PUP = 12,
    ARGUS = 13,
    EMCON = 14,
    XNET = 15,
    CHAOS = 16,
    UDP = 17,
    MUX = 18,
    DcnMeas = 19,
    HMP = 20,
    PRM = 21,
    XnsIdp = 22,
    Trunk1 = 23,
    Trunk2 = 24,
    Leaf1 = 25,
    Leaf2 = 26,
    RDP = 27,
    IRTP = 28,
    IsoTp4 = 29,
    NETBLT = 30,
    MfeNsp = 31,
    MeritInp = 32,
    DCCP = 33,
    _3PC = 34,
    IDPR = 35,
    XTP = 36,
    DDP = 37,
    IdprCmtp = 38,
    TPPlusPlus = 39,
    IL = 40,
    IPv6 = 41,
    SDRP = 42,
    Ipv6Route = 43,
    Ipv6Frag = 44,
    IDRP = 45,
    RSVP = 46,
    GRE = 47,
    DSR = 48,
    BNA = 49,
    ESP = 50,
    AH = 51,
    INlsp = 52,
    SWIPE = 53,
    NARP = 54,
    MinIpv4 = 55,
    TLSP = 56,
    SKIP = 57,
    Ipv6Icmp = 58,
    Ipv6NoNxt = 59,
    Ipv6Opts = 60,

    AnyHostInternalProtocol = 61,

    CFTP = 62,

    AnyLocalNetwork = 63,

    SatExpak = 64,
    KRYPTOLAN = 65,
    RVD = 66,
    IPPC = 67,

    AnyDistributedFileSystem = 68,

    SatMon = 69,
    VISA = 70,
    IPCV = 71,
    CPNX = 72,
    CPHB = 73,
    WSN = 74,
    PVP = 75,
    BrSatMon = 76,
    SunNd = 77,
    WbMon = 78,
    WbExpak = 79,
    IsoIp = 80,
    VMTP = 81,
    SecureVmtp = 82,
    VINES = 83,
    IPTM = 84,
    NsfnetIgp = 85,
    DGP = 86,
    TCF = 87,
    EIGRP = 88,
    OSPFIGP = 89,
    SpriteRpc = 90,
    LARP = 91,
    MTP = 92,
    Ax25 = 93,
    IPIP = 94,
    MICP = 95,
    SccSp = 96,
    ETHERIP = 97,
    ENCAP = 98,

    AnyPrivateEncryptionScheme = 99,

    GMTP = 100,
    IFMP = 101,
    PNNI = 102,
    PIM = 103,
    ARIS = 104,
    SCPS = 105,
    QNX = 106,
    AN = 107,
    IPComp = 108,
    SNP = 109,
    CompaqPeer = 110,
    IpxInIp = 111,
    VRRP = 112,
    PGM = 113,

    AnyZeroHopProtocol = 114,

    L2TP = 115,
    DDX = 116,
    IATP = 117,
    STP = 118,
    SRP = 119,
    UTI = 120,
    SMP = 121,
    SM = 122,
    PTP = 123,
    IsisOverIpv4 = 124,
    FIRE = 125,
    CRTP = 126,
    CRUDP = 127,
    SSCOPMCE = 128,
    IPLT = 129,
    SPS = 130,
    PIPE = 131,
    SCTP = 132,
    FC = 133,
    RsvpE2eIgnore = 134,
    MobilityHeader = 135,
    UDPLite = 136,
    MplsInIp = 137,
    Manet = 138,
    HIP = 139,
    Shim6 = 140,
    WESP = 141,
    ROHC = 142,
    Ethernet = 143,
    AGGFRAG = 144,
    NSH = 145,
    Homa = 146,
    BitEmu = 147,

    #[num_enum(alternatives = [148..252])]
    Unassigned = 252,

    Exp1 = 253,
    Exp2 = 254,
    Reserved = 255,
}

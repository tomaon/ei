// @see
//  http://erlang.org/doc/apps/erts/erl_ext_dist.html
//  http://erlang.org/doc/apps/erts/erl_dist_protocol.html

// -- ~/lib/erl_interface/include/ei.h --

pub const ERL_SMALL_INTEGER_EXT   :u8 = 0x61; //  97: 'a'
pub const ERL_INTEGER_EXT         :u8 = 0x62; //  98: 'b'
//  const ERL_FLOAT_EXT           :u8 = 0x63; //  99: 'c'
pub const NEW_FLOAT_EXT           :u8 = 0x46; //  70: 'F'
pub const ERL_ATOM_EXT            :u8 = 0x64; // 100: 'd'
//  const ERL_SMALL_ATOM_EXT      :u8 = 0x73; // 115: 's'
pub const ERL_ATOM_UTF8_EXT       :u8 = 0x76; // 118: 'v'
pub const ERL_SMALL_ATOM_UTF8_EXT :u8 = 0x77; // 119: 'w'
//  const ERL_REFERENCE_EXT       :u8 = 0x65; // 101: 'e'
//  const ERL_NEW_REFERENCE_EXT   :u8 = 0x72; // 114: 'r'
pub const ERL_NEWER_REFERENCE_EXT :u8 = 0x5a; //  90: 'Z'
//  const ERL_PORT_EXT            :u8 = 0x66; // 102: 'f'
pub const ERL_NEW_PORT_EXT        :u8 = 0x59; //  89: 'Y'
//  const ERL_PID_EXT             :u8 = 0x67; // 103: 'g'
pub const ERL_NEW_PID_EXT         :u8 = 0x58; //  88: 'X'
pub const ERL_SMALL_TUPLE_EXT     :u8 = 0x68; // 104: 'h'
pub const ERL_LARGE_TUPLE_EXT     :u8 = 0x69; // 105: 'i'
pub const ERL_NIL_EXT             :u8 = 0x6a; // 106: 'j'
pub const ERL_STRING_EXT          :u8 = 0x6b; // 107: 'k'
pub const ERL_LIST_EXT            :u8 = 0x6c; // 108: 'l'
pub const ERL_BINARY_EXT          :u8 = 0x6d; // 109: 'm'
//  const ERL_BIT_BINARY_EXT      :u8 = 0x4d; //  77: 'M'
pub const ERL_SMALL_BIG_EXT       :u8 = 0x6e; // 110: 'n'
//  const ERL_LARGE_BIG_EXT       :u8 = 0x6f; // 111: 'o',
//  const ERL_NEW_FUN_EXT         :u8 = 0x70; // 112: 'p'
pub const ERL_MAP_EXT             :u8 = 0x74; // 116: 't'
//  const ERL_FUN_EXT             :u8 = 0x75; // 117: 'u'
//  const ERL_EXPORT_EXT          :u8 = 0x71; // 113: 'q'
pub const ERL_V4_PORT_EXT         :u8 = 0x78; // 120: 'x'

//  const ERL_NEW_CACHE           :u8 = 0x4e; //  78: 'N'
//  const ERL_CACHED_ATOM         :u8 = 0x43; //  67: 'C'

//  const ERL_LINK                :u8 = 0x01; //   1
pub const ERL_SEND                :u8 = 0x02; //   2
pub const ERL_EXIT                :u8 = 0x03; //   3
//  const ERL_UNLINK              :u8 = 0x04; //   4
//  const ERL_NODE_LINK           :u8 = 0x05; //   5
pub const ERL_REG_SEND            :u8 = 0x06; //   6
//  const ERL_GROUP_LEADER        :u8 = 0x07; //   7
//  const ERL_EXIT2               :u8 = 0x08; //   8
//  const ERL_PASS_THROUGH        :u8 = 0x70; // 112: 'p'

pub const ERL_SEND_TT             :u8 = 0x0c; //  12
pub const ERL_EXIT_TT             :u8 = 0x0d; //  13
pub const ERL_REG_SEND_TT         :u8 = 0x10; //  16
//  const ERL_EXIT2_TT            :u8 = 0x12; //  18
//  const ERL_MONITOR_P           :u8 = 0x13; //  19
//  const ERL_DEMONITOR_P         :u8 = 0x14; //  20
//  const ERL_MONITOR_P_EXIT      :u8 = 0x15; //  21

pub const MAXATOMLEN           :usize = 255 + 1;
pub const MAXATOMLEN_UTF8      :usize = 255*4 + 1;
//  const MAXNODELEN           :usize = MAXATOMLEN;
//  const EI_MAXHOSTNAMELEN    :usize = MAXATOMLEN - 2;
//  const EI_MAXALIVELEN       :usize = MAXATOMLEN - 2;
//  const EI_MAX_COOKIE_SIZE   :usize = 512;

// -- ~/lib/erl_interface/src/connect/ei_connect_int.h --

//  const DFLAG_PUBLISHED           :u64 = 0x00000001;
//  const DFLAG_ATOM_CACHE          :u64 = 0x00000002;
//  const DFLAG_EXTENDED_REFERENCES :u64 = 0x00000004;
//  const DFLAG_DIST_MONITOR        :u64 = 0x00000008;
//  const DFLAG_FUN_TAGS            :u64 = 0x00000010;
//  const DFLAG_NEW_FUN_TAGS        :u64 = 0x00000080;
//  const DFLAG_EXTENDED_PIDS_PORTS :u64 = 0x00000100;
//  const DFLAG_EXPORT_PTR_TAG      :u64 = 0x00000200;
//  const DFLAG_BIT_BINARIES        :u64 = 0x00000400;
//  const DFLAG_NEW_FLOATS          :u64 = 0x00000800;
//  const DFLAG_SMALL_ATOM_TAGS     :u64 = 0x00004000;
//  const DFLAG_UTF8_ATOMS          :u64 = 0x00010000;
//  const DFLAG_MAP_TAG             :u64 = 0x00020000;
//  const DFLAG_BIG_CREATION        :u64 = 0x00040000;
//  const DFLAG_HANDSHAKE_23        :u64 = 0x01000000;
//  const DFLAG_UNLINK_ID           :u64 = 0x02000000;
//  const DFLAG_MANDATORY_25_DIGEST :u64 = 0x04000000;
//  const DFLAG_RESERVED            :u64 = 0xf8000000;
//  const DFLAG_NAME_ME             :u64 = 0x2 << 32;
//  const DFLAG_V4_NC               :u64 = 0x4 << 32;

// -- ~/lib/erl_interface/src/connect/ei_connect.c --

//  const COOKIE_FILE : &'static str = ".erlang.cookie";

// -- ~/lib/erl_interface/src/epmd/ei_epmd.h --

//  const EI_DIST_5            :u16 = 5; // OTP R4 - 22
//  const EI_DIST_6            :u16 = 6; // OTP 23

//  const EI_DIST_HIGH         :u16 = EI_DIST_6;
//  const EI_DIST_LOW          :u16 = EI_DIST_5;

//  const EPMD_PORT            :u16 = 4369;

//  const EPMDBUF            :usize = 512;

//  const EI_MYPROTO            :u8 = 0; // tcp/ip

//  const EI_EPMD_ALIVE2_REQ    :u8 = 0x78; // 120: 'x'
//  const EI_EPMD_ALIVE2_RESP   :u8 = 0x79; // 121: 'y'
//  const EI_EPMD_ALIVE2_X_RESP :u8 = 0x76; // 118: 'v'
//  const EI_EPMD_PORT2_REQ     :u8 = 0x7a; // 122: 'z'
//  const EI_EPMD_PORT2_RESP    :u8 = 0x77; // 119: 'w'
//  const EI_EPMD_STOP_REQ      :u8 = 0x73; // 115: 's'

// -- ~/lib/erl_interface/src/misc/eiext.h --

pub const ERL_VERSION_MAGIC :u8 = 0x83; // 131

// -- --

//  const EI_HIDDEN_NODE        :u8 = 0x68; // 104: 'h'
//  const EI_SUCCESS            :i8 = 0x00;

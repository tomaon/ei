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
pub const ERL_NEW_REFERENCE_EXT   :u8 = 0x72; // 114: 'r'
pub const ERL_NEWER_REFERENCE_EXT :u8 = 0x5a; //  90: 'Z'
pub const ERL_PORT_EXT            :u8 = 0x66; // 102: 'f'
pub const ERL_NEW_PORT_EXT        :u8 = 0x59; //  89: 'Y'
pub const ERL_PID_EXT             :u8 = 0x67; // 103: 'g'
pub const ERL_NEW_PID_EXT         :u8 = 0x58; //  88: 'X'
pub const ERL_SMALL_TUPLE_EXT     :u8 = 0x68; // 104: 'h'
pub const ERL_LARGE_TUPLE_EXT     :u8 = 0x69; // 105: 'i'
pub const ERL_NIL_EXT             :u8 = 0x6a; // 106: 'j'
pub const ERL_STRING_EXT          :u8 = 0x6b; // 107: 'k'
pub const ERL_LIST_EXT            :u8 = 0x6c; // 108: 'l'
//  const ERL_BINARY_EXT          :u8 = 0x6d; // 109: 'm'
pub const ERL_SMALL_BIG_EXT       :u8 = 0x6e; // 110: 'n'
//  const ERL_LARGE_BIG_EXT       :u8 = 0x6f; // 111: 'o',
//  const ERL_NEW_FUN_EXT         :u8 = 0x70; // 112: 'p'
pub const ERL_MAP_EXT             :u8 = 0x74; // 116: 't'
//  const ERL_FUN_EXT             :u8 = 0x75; // 117: 'u'
//  const L_NEW_CACHE             :u8 = 0x4e; //  78: 'N'
//  const L_CACHED_ATOM           :u8 = 0x43; //  67: 'C'

//  const ERL_LINK                :u8 = 0x01; //   1
pub const ERL_SEND                :u8 = 0x02; //   2
pub const ERL_EXIT                :u8 = 0x03; //   3
//  const ERL_UNLINK              :u8 = 0x04; //   4
//  const ERL_NODE_LINK           :u8 = 0x05; //   5
pub const ERL_REG_SEND            :u8 = 0x06; //   6
//  const ERL_GROUP_LEADER        :u8 = 0x07; //   7
//  const ERL_EXIT2               :u8 = 0x08; //   8
pub const ERL_PASS_THROUGH        :u8 = 0x70; // 112: 'p'

pub const ERL_SEND_TT             :u8 = 0x0c; //  12
pub const ERL_EXIT_TT             :u8 = 0x0d; //  13
pub const ERL_REG_SEND_TT         :u8 = 0x10; //  16
//  const ERL_EXIT2_TT            :u8 = 0x12; //  18
//  const ERL_MONITOR_P           :u8 = 0x13; //  19
//  const ERL_DEMONITOR_P         :u8 = 0x14; //  20
//  const ERL_MONITOR_P_EXIT      :u8 = 0x15; //  21

pub const EI_MAXHOSTNAMELEN    :usize =  64;
pub const EI_MAXALIVELEN       :usize =  63;
pub const EI_MAX_COOKIE_SIZE   :usize = 512;
pub const MAXATOMLEN           :usize = 255 + 1;
pub const MAXATOMLEN_UTF8      :usize = 255*4 + 1;
pub const MAXNODELEN           :usize = EI_MAXALIVELEN+1+EI_MAXHOSTNAMELEN;

// -- ~/lib/erl_interface/src/connect/ei_connect_int.h --

//  const DFLAG_PUBLISHED           :u32 = 1 <<  0;
//  const DFLAG_ATOM_CACHE          :u32 = 1 <<  1;
pub const DFLAG_EXTENDED_REFERENCES :u32 = 1 <<  2;
pub const DFLAG_DIST_MONITOR        :u32 = 1 <<  3;
pub const DFLAG_FUN_TAGS            :u32 = 1 <<  4;
pub const DFLAG_NEW_FUN_TAGS        :u32 = 1 <<  7;
pub const DFLAG_EXTENDED_PIDS_PORTS :u32 = 1 <<  8;
pub const DFLAG_NEW_FLOATS          :u32 = 1 << 11;
pub const DFLAG_SMALL_ATOM_TAGS     :u32 = 1 << 14;
pub const DFLAG_UTF8_ATOMS          :u32 = 1 << 16;
pub const DFLAG_MAP_TAG             :u32 = 1 << 17;
pub const DFLAG_BIG_CREATION        :u32 = 1 << 18;

// -- ~/lib/erl_interface/src/misc/eiext.h --

pub const ERL_VERSION_MAGIC :u8 = 0x83; // 131

// -- /usr/include/errno.h --

pub const EINTR        :i32 =  4; // Interrupted system call
pub const EIO          :i32 =  5; // Input/output error
pub const EINVAL       :i32 = 22; // Invalid argument
pub const EDOM         :i32 = 33; // Numerical argument out of domain
pub const ERANGE       :i32 = 34; // Result too large
//  const EMSGSIZE     :i32 = 40; // Message too long
//  const ETIMEDOUT    :i32 = 60; // Operation timed out
//  const EHOSTUNREACH :i32 = 65; // No route to host

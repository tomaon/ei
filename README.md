# [erl_interface][1] for Rust

## Example

rust:
```bash
$ epmd -daemon
$
$ cargo run --example echo
local: V6([::1]:3456)
epmd : (3456, 5)
```

erlang:
```erlang
$ erl -sname e1 -proto_dist inet6_tcp
Eshell V8.0  (abort with ^G)
(e1@x)1> {any, 'r1@localhost'} ! 123. % u64 only
123
(e1@x)2> flush().
Shell got {ok,123}
ok
```

## License

Apache-2.0

[1]: http://erlang.org/doc/apps/erl_interface/index.html

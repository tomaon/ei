# [erl_interface][1] for Rust

## build

```bash
cargo build
```

## Example

### cnode:

rust:
```bash
$ epmd -daemon
$
$ cargo run --example echo
local: V6([::1]:3456)
epmd : (3456, 5)
```

erlang:
```bash
% erl -sname e1 -proto_dist inet6_tcp
Eshell V8.0  (abort with ^G)
(e1@x)1>
```

```erlang
(e1@x)1> {any, 'r1@localhost'} ! 123. % u64 only
123
(e1@x)2> flush().
Shell got {ok,123}
ok
```

### port:

rust:
```bash
$ cargo build --example port
...
$ cd examples
$ erl
Eshell V8.0  (abort with ^G)
1>
```

erlang:
```erlang
1> c(port).
{ok,port}
2> {ok,P} = port:start_link().
{ok,<0.63.0>}
3> port:foo(P, 3).
{ok,4}
4> port:bar(P, 3).
{ok,6}
5> port:baz(P, 3).
{error,badarg}
6> port:stop(P).
ok
```

## License

Apache-2.0

[1]: http://erlang.org/doc/apps/erl_interface/index.html

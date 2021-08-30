# [erl_interface][1] for Rust

## TODO
- improper list
- cnode

## build
- rust: 1.54.0
- erlang: 24.0.5

```bash
cargo build
```

## Example

### port:

rust:
```bash
$ cargo build --example port
...
$ cd examples
$ erl
Eshell V12.0.3  (abort with ^G)
1>
```

erlang:
```erlang
1> c(port).
{ok,port}
2> {ok,P} = port:start_link().
{ok,<0.86.0>}
3> port:add(P, 1, 2).
{ok,3}
4> port:sub(P, 3, 1).
{ok,2}
5> port:mul(P, 2, 3).
{error,undef}
6> port:stop(P).
ok
```

## License

Apache-2.0

[1]: http://erlang.org/doc/apps/erl_interface/index.html

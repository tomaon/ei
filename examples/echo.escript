#!/usr/bin/env escript
%% -*- erlang -*-
%%! -sname e1 -proto_dist inet6_tcp

echo(Args, Timeout) ->
    {any, 'r1@localhost'} ! Args,
    receive
        Any ->
            Any
    after
        Timeout ->
            {error, timeout}
    end.

run(num) ->
    L = [
         {"i64-err", -9223372036854775808},
         {"i64",     -9223372036854775807},
         {"i32-err",          -2147483648},
         {"i32",              -2147483647},
         {"i16",                   -32768},
         {"i8",                      -128},
         {"u8",                         0},
         {"i8",                       127},
         {"u8",                       255},
         {"i16",                    32767},
         {"u16",                    65535},
         {"i32",               2147483647},
         {"u32",               4294967295},
         {"i64",      9223372036854775807},
         {"u64",     18446744073709551615}
        ],
    [ io:format("~p: ~p, ~p~n", [T, {ok,E} == echo(E, 100), E]) || {T,E} <- L ];
run(_) ->
    ok.

main(_) ->
    run(num).

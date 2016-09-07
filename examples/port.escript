#!/usr/bin/env escript
%% -*- erlang -*-

main(_) ->
    case port:start_link() of
        {ok, P} ->
            S = self(),
            spawn(fun() -> S ! {"foo(3):", port:foo(P,3)} end),
            spawn(fun() -> S ! {"bar(3):", port:bar(P,3)} end),
            spawn(fun() -> S ! {"baz(3):", port:baz(P,3)} end),
            recv(3, 5000),
            ok = port:stop(P);
        Other ->
            io:format("other: ~p~n", [Other])
    end.

recv(0, _) ->
    ok;
recv(N, T) ->
    receive
        Any ->
            io:format("~p~n", [Any])
    after
        T ->
            {error, timeout}
    end,
    recv(N - 1, T).

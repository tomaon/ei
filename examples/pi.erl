-module(pi).

-export([calc_pi/2]).

%% erl -sname e1 -proto_dist inet6_tcp
%% Eshell V8.0  (abort with ^G)
%% (e1@x)1> c(pi).
%% {ok,pi}
%% (e1@x)2> timer:tc(pi, calc_pi, [1000000000,10]).
%% {16735480,{ok,3.141592655589816}}

calc_pi(N, NumThreads) ->
    {any, 'r1@localhost'} ! {N, NumThreads},
    receive
        Any ->
            Any
    end.

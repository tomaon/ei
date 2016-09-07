-module(cnode).

-export([calc_pi/2]).

calc_pi(N, NumThreads) ->
    {any, 'r1@localhost'} ! {N, NumThreads},
    receive
        Any ->
            Any
    end.

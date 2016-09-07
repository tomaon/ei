-module(port).

%% -- public --
-export([start_link/0, stop/1]).
-export([foo/2, bar/2, baz/2, pi/3]).

%% -- behaviour: gen_server --
-behaviour(gen_server).
-export([init/1, terminate/2, code_change/3,
         handle_call/3, handle_cast/2, handle_info/2]).

%% -- internal --
-record(state, {
          port :: port()
         }).

%% == public ==

-spec start_link() -> {ok,pid()}|{error,_}.
start_link() ->
    L = [
         {spawn_executable, <<"../target/debug/examples/port">>},
         [
          {packet, 2},
          %% {cd, <<"/tmp">>}, => crash, TODO
          %% {env, [
          %%        {"RUST_LOG", "trace"}
          %%       ]},
          %% {args, [<<"-a">>, <<"-b">>, <<"-c">>]},
          %% {arg0, <<"xx">>},
          use_stdio,
          binary
         ]
        ],
    start_link([{open_port,L}], []).

-spec stop(pid()) -> ok.
stop(Pid)
  when is_pid(Pid) ->
    gen_server:call(Pid, stop).


-spec foo(pid(),integer()) -> {ok,integer()}|{error,_}.
foo(Pid, Long)
  when is_pid(Pid), is_integer(Long) ->
    call(Pid, $f, Long, timer:seconds(3)).

-spec bar(pid(),integer()) -> {ok,integer()}|{error,_}.
bar(Pid, Long)
  when is_pid(Pid), is_integer(Long) ->
    call(Pid, $b, Long, timer:seconds(3)).

-spec baz(pid(),integer()) -> {ok,integer()}|{error,_}.
baz(Pid, Long)
  when is_pid(Pid), is_integer(Long) ->
    call(Pid, $z, Long, timer:seconds(3)).

-spec pi(pid(),integer(),integer()) -> {ok,float()}|{error,_}.
pi(Pid, N, NumThreads)
  when is_pid(Pid), is_integer(N), is_integer(NumThreads) ->
    call(Pid, N, NumThreads, infinity).

%% == private ==

start_link(Args, Options) ->
    case gen_server:start_link(?MODULE, [], Options) of
        {ok, Pid} ->
            case gen_server:call(Pid, {setup,Args}) of
                ok ->
                    {ok, Pid};
                {error, Reason} ->
                    ok = stop(Pid),
                    {error, Reason}
            end
    end.

call(Pid, Char, Int, Timeout) ->
    gen_server:call(Pid, {port_command,Char,Int}, Timeout).

%% == behaviour: gen_server ==

init(Args) ->
    setup(Args).

terminate(_Reason, State) ->
    cleanup(State).

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.

handle_call({port_command,C,I}, From, #state{port=P}=S) ->
    true = port_command(P, term_to_binary({C,I,From})),
    {noreply, S};
handle_call({setup,Args}, _From, State) ->
    setup(Args, State);
handle_call(stop, _From, #state{port=P}=S) ->
    true = port_command(P, <<>>),
    {stop, normal, ok, S}.

handle_cast(_Request, State) ->
    {stop, enotsup, State}.

handle_info({P,{data,B}}, #state{port=P}=S) ->
    Tuple = binary_to_term(B),
    _ = apply(gen_server, reply, delete_element(size(Tuple),Tuple)),
    {noreply, S};
handle_info({'EXIT',P,Reason}, #state{port=P}=S) ->
    {stop, {port_close,Reason}, S#state{port = undefined}};
handle_info({'EXIT',_Pid,Reason}, State) ->
    {stop, Reason, State}.

%% == intarnal ==

cleanup(#state{port=P}=S)
  when P =/= undefined ->
    true = port_close(P),
    cleanup(S#state{port = undefined});
cleanup(#state{}) ->
    no_return.

setup([]) ->
    _ = process_flag(trap_exit, true),
    {ok, #state{}}.

setup([{open_port,A}|T], #state{port=undefined}=S) ->
    try apply(erlang, open_port, A) of
        Port ->
            setup(T, S#state{port = Port})
    catch
        error:Reason ->
            {reply, {error,Reason}, S}
    end;
setup([], State) ->
    {reply, ok, State}.


delete_element(Index, Tuple) ->
    Term = element(Index, Tuple),
    [Term, erlang:delete_element(Index,Tuple)].

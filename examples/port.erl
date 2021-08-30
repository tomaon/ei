-module(port).

%% -- public --
-export([start_link/0, stop/1]).
-export([add/3, sub/3, mul/3]).

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

-spec add(pid(),integer(),integer()) -> {ok,integer()}|{error,_}.
add(Pid, Int1, Int2)
  when is_pid(Pid), is_integer(Int1), is_integer(Int2) ->
    gen_server:call(Pid, {port_command, $a, Int1, Int2}, timer:seconds(3)).

-spec sub(pid(),integer(),integer()) -> {ok,integer()}|{error,_}.
sub(Pid, Int1, Int2)
  when is_pid(Pid), is_integer(Int1), is_integer(Int2) ->
    gen_server:call(Pid, {port_command, $s, Int1, Int2}, timer:seconds(3)).

-spec mul(pid(),integer(),integer()) -> {ok,integer()}|{error,_}.
mul(Pid, Int1, Int2)
  when is_pid(Pid), is_integer(Int1), is_integer(Int2) ->
    gen_server:call(Pid, {port_command, $m, Int1, Int2}, timer:seconds(3)).

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

%% == behaviour: gen_server ==

init(Args) ->
    setup(Args).

terminate(_Reason, State) ->
    cleanup(State).

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.

% Tag: [H|T], improper list -> tuple
handle_call({port_command,C,I1,I2}, {F,[H|T]}, #state{port=P}=S) ->
    true = port_command(P, term_to_binary({C,I1,I2,F,H,T})),
    {noreply, S};
handle_call({setup,Args}, _From, State) ->
    setup(Args, State);
handle_call(stop, _From, #state{port=P}=S) ->
    true = port_command(P, <<>>),
    {stop, normal, ok, S}.

handle_cast(_Request, State) ->
    {stop, enotsup, State}.

handle_info({P,{data,B}}, #state{port=P}=S) ->
    {R,V,F,H,T} = binary_to_term(B),
    ok = gen_server:reply({F,[H|T]}, {R,V}),
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

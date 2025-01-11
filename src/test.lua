print("Hey there!", _VERSION, "running in Godot!")
a = "uwu"
warn(a)
print(Vector3int16.new(32768, 10, -11))
print(Instance)
print(Instance.new("Model"))

local coro = coroutine.create(function(owo)
    print("coroutine!! :3",owo)
end)
coroutine.resume(coro,"uwu")
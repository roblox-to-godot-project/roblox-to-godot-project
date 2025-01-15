print("Hey there!", _VERSION, "running in Godot!")
a = "uwu"
warn(a)
print(Vector3int16.new(32768, 10, -11))
print(Instance)
local i = Instance.new("Model")
local child = Instance.new("Model")
print(i)
i.Name = "uwu"
print(i)

i.ChildAdded:Connect(function(i) print("ChildAdded", i) end)
i.ChildRemoved:Connect(function(i) print("ChildRemoved", i) end)

child.Parent = i

print("Children")
for i,v in ipairs(i:GetChildren()) do
    print(i, v)
end

child:Destroy()

local coro = coroutine.create(function(owo)
    print("coroutine!! :3",owo)
end)
coroutine.resume(coro,"uwu")
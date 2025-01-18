print("Hey there!", _VERSION, "running in Godot!")
a = "uwu"
warn(a)
print(Vector3int16.new(32768, 10, -11))
print(Instance)
local i = Instance.new("Model")
local child = Instance.new("Model")
print(i)
i.Name = "ParentInstance"
child.Name = "ChildInstance"
print(i)

i.ChildAdded:Connect(function(i) print("ChildAdded", i) end)
i.ChildRemoved:Connect(function(i) print("ChildRemoved", i) end)
child.Destroying:Connect(function() print("Destroying", child) end)
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
print(getmetatable(CFrame.new()))
local cframe = CFrame.new()
local cframe_rel = CFrame.new(Vector3.new(5, 4, 9))
local cframe_look_at = CFrame.new(cframe_rel.Position, Vector3.new(5, 0, 9))
local cframe_look_along = CFrame.lookAlong(Vector3.new(0, 1, 0), Vector3.new(0, -1, 1).Unit)
print(cframe, cframe_rel, cframe_look_at, cframe_look_along)
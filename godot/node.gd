extends Node

func _ready() -> void:
	$RobloxVM.push_code(r"""
local model = Instance.new("Model")
print(model, game)
""")

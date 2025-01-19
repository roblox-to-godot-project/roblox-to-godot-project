#[derive(Instance)]
#[instance(hierarchy=[ServiceProvider])]
#[method(some_method = "method_name")]
struct UrStruct {
    #[property(name = "YourPropertyName")]
    property_name: PropertyType
}
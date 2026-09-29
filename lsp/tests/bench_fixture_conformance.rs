include!("../benches/fixtures.rs");

use ktav::Value;

#[test]
fn small_bench_fixture_is_a_current_syntax_object() {
    let doc = small();
    let value = ktav::parse(&doc).expect("small bench fixture must parse");
    let Value::Object(root) = value else {
        panic!("small bench fixture must have an object root");
    };

    assert!(matches!(root.get("name_0"), Some(Value::String(_))));
    assert!(matches!(root.get("port_1"), Some(Value::Integer(_))));
    assert!(matches!(root.get("ratio_2"), Some(Value::Float(_))));
    assert!(matches!(root.get("flag_3"), Some(Value::Bool(_))));
    assert!(matches!(root.get("label_4"), Some(Value::String(_))));
    let Some(Value::Object(service)) = root.get("service") else {
        panic!("dotted service keys must form an object");
    };
    let Some(Value::Object(service_5)) = service.get("5") else {
        panic!("service.5 must be an object");
    };
    assert!(matches!(service_5.get("host"), Some(Value::String(_))));
    let Some(Value::Object(service_6)) = service.get("6") else {
        panic!("service.6 must be an object");
    };
    assert!(matches!(service_6.get("port"), Some(Value::Integer(_))));
    assert!(doc.contains("## section 0\n"));
    let Some(Value::Object(obj)) = root.get("obj_8") else {
        panic!("obj_8 must be an object");
    };
    assert!(matches!(obj.get("inner_a"), Some(Value::Integer(_))));
    assert!(matches!(obj.get("inner_b"), Some(Value::Float(_))));
    assert!(matches!(obj.get("inner_c"), Some(Value::String(_))));
    assert!(matches!(root.get("list_9"), Some(Value::Array(_))));
    assert!(matches!(root.get("doc_10"), Some(Value::String(_))));
    assert!(matches!(root.get("tag_11"), Some(Value::String(_))));
}

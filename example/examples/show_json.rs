use example::{Opt, SubCommands, EnumType};

fn main() {
    // Example 1: With Sub1
    let opt1 = Opt {
        string_field: Some("test".to_string()),
        string_default: "default.txt".to_string(),
        counter_field: 2,
        bool_field: true,
        int_field: 42,
        enum_field: EnumType::OptionB,
        vec_field: vec!["item1".to_string(), "item2".to_string()],
        uint_field: 10,
        optional_field: None,
        flag_field: false,
        subcommand: Some(SubCommands::Sub1 {
            arg1: "hello world".to_string(),
            flag1: true,
        }),
    };

    println!("Example with Sub1:");
    println!("{}\n", serde_json::to_string_pretty(&opt1).unwrap());

    // Example 2: With Sub2
    let opt2 = Opt {
        string_field: None,
        string_default: "default.txt".to_string(),
        counter_field: 0,
        bool_field: false,
        int_field: 42,
        enum_field: EnumType::OptionA,
        vec_field: vec![],
        uint_field: 10,
        optional_field: Some("value".to_string()),
        flag_field: true,
        subcommand: Some(SubCommands::Sub2 {
            str_arg: "template".to_string(),
            num_arg: 5,
        }),
    };

    println!("Example with Sub2:");
    println!("{}\n", serde_json::to_string_pretty(&opt2).unwrap());

    // Example 3: With Sub3
    let opt3 = Opt {
        string_field: Some("input.txt".to_string()),
        string_default: "output.txt".to_string(),
        counter_field: 3,
        bool_field: true,
        int_field: 100,
        enum_field: EnumType::OptionD,
        vec_field: vec!["a".to_string(), "b".to_string()],
        uint_field: 20,
        optional_field: None,
        flag_field: false,
        subcommand: Some(SubCommands::Sub3 {
            pos_arg: "file.json".to_string(),
            flag_arg: true,
        }),
    };

    println!("Example with Sub3:");
    println!("{}\n", serde_json::to_string_pretty(&opt3).unwrap());

    // Example 4: No subcommand
    let opt4 = Opt {
        string_field: None,
        string_default: "default.txt".to_string(),
        counter_field: 0,
        bool_field: false,
        int_field: 42,
        enum_field: EnumType::OptionA,
        vec_field: vec![],
        uint_field: 10,
        optional_field: None,
        flag_field: false,
        subcommand: None,
    };

    println!("Example with no subcommand:");
    println!("{}", serde_json::to_string_pretty(&opt4).unwrap());
}

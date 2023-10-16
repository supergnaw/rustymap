use std::process::exit;
use regex::Regex;
use crate::tag::*;

#[derive(Debug)]
pub struct NBT {
    pub(crate) tags: Tag,
    bytes: Vec<u8>,
}

impl NBT {
    pub fn new(bytes: &Vec<u8>) -> NBT {
        NBT {
            tags: Tag::new(bytes.clone()),
            bytes: bytes.clone(),
        }
    }

    pub fn test() {
        let mut test_tags: Vec<Vec<u8>> = vec![];
        // Test_End
        let test_end = vec![0];
        test_tags.push(test_end.clone());
        // Test_Byte
        let test_byte = vec![1, 0, 9, 84, 101, 115, 116, 95, 66, 121, 116, 101, 1];
        test_tags.push(test_byte.clone());
        // Test_Short
        let test_short = vec![2, 0, 10, 84, 101, 115, 116, 95, 83, 104, 111, 114, 116, 1, 2];
        test_tags.push(test_short.clone());
        // Test_Int
        let test_int = vec![3, 0, 8, 84, 101, 115, 116, 95, 73, 110, 116, 1, 2, 3, 4];
        test_tags.push(test_int.clone());
        // test_long
        let test_long = vec![4, 0, 9, 84, 101, 115, 116, 95, 76, 111, 110, 103, 1, 2, 3, 4, 5, 6, 7, 8];
        test_tags.push(test_long.clone());
        // Test_Float
        let test_float = vec![5, 0, 10, 84, 101, 115, 116, 95, 70, 108, 111, 97, 116, 1, 2, 3, 4];
        test_tags.push(test_float.clone());
        // Test_Double
        let test_double = vec![6, 0, 11, 84, 101, 115, 116, 95, 68, 111, 117, 98, 108, 101, 1, 2, 3, 4, 5, 6, 7, 8];
        test_tags.push(test_double.clone());
        // Test_Byte_Array
        let test_byte_array = vec![7, 0, 15, 84, 101, 115, 116, 95, 66, 121, 116, 101, 95, 65, 114, 114, 97, 121, 0, 0, 0, 4, 1, 2, 3, 4];
        test_tags.push(test_byte_array.clone());
        // Test_String
        let test_string = vec![8, 0, 11, 84, 101, 115, 116, 95, 83, 116, 114, 105, 110, 103, 0, 3, 102, 111, 111];
        test_tags.push(test_string.clone());
        // Test_List
        let test_list_byte = vec![9, 0, 9, 84, 101, 115, 116, 95, 76, 105, 115, 116, 1, 0, 0, 0, 8, 1, 2, 3, 4, 5, 6, 7, 8];
        test_tags.push(test_list_byte.clone());
        let test_list_short = vec![9, 0, 9, 84, 101, 115, 116, 95, 76, 105, 115, 116, 2, 0, 0, 0, 4, 1, 2, 3, 4, 5, 6, 7, 8];
        test_tags.push(test_list_short.clone());
        let test_list_int = vec![9, 0, 9, 84, 101, 115, 116, 95, 76, 105, 115, 116, 3, 0, 0, 0, 2, 1, 2, 3, 4, 5, 6, 7, 8];
        test_tags.push(test_list_int.clone());
        let test_list_long = vec![9, 0, 9, 84, 101, 115, 116, 95, 76, 105, 115, 116, 4, 0, 0, 0, 1, 1, 2, 3, 4, 5, 6, 7, 8];
        test_tags.push(test_list_long.clone());
        // Test_Int_Array
        let test_int_array = vec![11, 0, 14, 84, 101, 115, 116, 95, 73, 110, 116, 95, 65, 114, 114, 97, 121, 0, 0, 0, 4, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
        test_tags.push(test_int_array);
        // Test_Long_Array
        let test_long_array = vec![12, 0, 15, 84, 101, 115, 116, 95, 76, 111, 110, 103, 95, 65, 114, 114, 97, 121, 0, 0, 0, 2, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8];
        test_tags.push(test_long_array);
        // Test_Compound
        let mut test_compound = vec![10, 0, 13, 84, 101, 115, 116, 95, 67, 111, 109, 112, 111, 117, 110, 100];
        test_compound.extend(&test_byte);
        test_compound.extend(&test_short);
        test_compound.extend(&test_int);
        test_compound.extend(&test_long);
        test_compound.extend(&test_float);
        test_compound.extend(&test_double);
        test_compound.extend(&test_byte_array);
        test_compound.extend(&test_string);
        test_compound.extend(&test_int_array);
        test_compound.extend(&test_long_array);
        test_compound.extend(&test_list_byte);
        test_compound.extend(&test_list_short);
        test_compound.extend(&test_list_int);
        test_compound.extend(&test_list_long);
        test_compound.extend(&test_end);
        test_tags.push(test_compound);

        println!("\n");
        for test_tag in test_tags {
            println!("bytes: {:?}", &test_tag);
            let test_tag = Tag::new(test_tag);
            println!("tag: {:?}\n", test_tag);
        }

        println!("If you see this message then all test bytes parsed without errors.");

        exit(0);
    }
}

pub fn camel_to_snake(input: &str) -> String {
    // if you say regex wrong you probably mispronounce gif too
    let re_snake = Regex::new(r"([A-Z])").unwrap();
    let re_collapse = Regex::new(r"_+").unwrap();

    // precede uppercase with underscore and convert to lowercase
    let snake_case = re_snake.replace_all(input, |caps: &regex::Captures| {
        format!("_{}", &caps[1].to_lowercase())
    });

    // collapse repeating underscores
    let snake_case = re_collapse.replace_all(&snake_case, "_").to_string();

    // trim leading and following underscores
    snake_case.trim_matches('_').to_string()
}

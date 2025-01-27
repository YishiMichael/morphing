pub(crate) fn read_from_stdin<T>() -> T
where
    T: serde::de::DeserializeOwned,
{
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    ron::de::from_str(&buf).unwrap()
}

pub(crate) fn write_to_stdout<T>(value: T)
where
    T: serde::Serialize,
{
    println!("{}", ron::ser::to_string(&value).unwrap());
}

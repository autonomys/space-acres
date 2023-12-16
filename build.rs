fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res\\windows\\space-acres.ico");
        res.compile().unwrap();
    }
}

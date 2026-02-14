fn main() {
    println!("cargo:rustc-link-lib=framework=WebKit");
    println!("cargo:rustc-link-lib=dispatch");
}


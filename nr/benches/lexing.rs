#![feature(test)]

extern crate test;

use test::Bencher;

use nreplops_tool::clojure::lex;

#[bench]
fn bench_hello(b: &mut Bencher) {
  const INPUT: &str = "hello";
  b.iter(|| {
    let _ = lex::lex(INPUT).unwrap();
  })
}

#[bench]
fn bench_complex_input(b: &mut Bencher) {
  const INPUT: &str = r#"
;; More complex input

(prn :foo{:bar    "Hello, world"
          :answer 42})
"#;
  b.iter(|| {
    lex::lex(INPUT).unwrap();
  })
}

use parser::Node;

struct Eval;

impl Eval {
    pub fn new() -> Self {
        Eval {}
    }

    fn calc_integer<F>(&self, f: &F, args: &[Node], node: &Node) -> Node
        where F: Fn(i64, i64) -> i64 {

        Node::Integer(
            args.iter().fold(None, |a, x| match x {
                &Node::Integer(i) => match a {
                    Some(aa) => Some(f(aa, i)),
                    None => Some(i)
                },
                // TODO: Don't copy
                &Node::List(_) => match &self.eval(x.clone()) {
                    &Node::Integer(i) => match a {
                        Some(aa) => Some(f(aa, i)),
                        None => Some(i)
                    },
                    _ => panic!("{:?} takes only an integer, but got {:?}", node, x),
                },
                _ => panic!("{:?} takes only an integer, but got {:?}", node, x)
            }
        ).unwrap_or(0))
    }

    pub fn eval(&self, node: Node) -> Node {
        match node {
            Node::Integer(_) => node,
            Node::Keyword(_) => panic!("Unexpected node: {:?}", node),
            Node::List(ref xs) => {
                let (hd, tl) = xs.split_first().unwrap();
                match hd {
                    &Node::Keyword(ref kwd) => 
                        match kwd.as_str() {
                            "+" => self.calc_integer(&|a, i| a + i, tl, &node),
                            "-" => self.calc_integer(&|a, i| a - i, tl, &node),
                            "*" => self.calc_integer(&|a, i| a * i, tl, &node),
                            "/" => self.calc_integer(&|a, i| a / i, tl, &node),
                            _ => panic!("Unexpected node: {:?}", node),
                        },
                    _ => panic!("Unexpected node: {:?}", node),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Eval;
    use parser::Node;

    #[test]
    fn eval() {
        assert_eq!(
            Node::Integer(42),
            Eval::new().eval(
                Node::List(
                    vec![
                        Node::Keyword(String::from("+")),
                        Node::Integer(2),
                        Node::Integer(40),
                    ]
                )
            )
        );

        assert_eq!(
            Node::Integer(42),
            Eval::new().eval(
                Node::List(
                    vec![
                        Node::Keyword(String::from("*")),
                        Node::Integer(6),
                        Node::List(
                            vec![
                                Node::Keyword(String::from("-")),
                                Node::Integer(42),
                                Node::Integer(35),
                            ]
                        ),
                    ]
                )
            )
        );
    }
}

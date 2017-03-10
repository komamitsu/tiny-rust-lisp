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

    fn car(&self, args: &[Node], node: &Node) -> Node {
        if args.len() == 1 {
            if let &Node::QuotedList(ref xs) = &args[0] {
                return match xs.split_first() {
                    // TODO: Don't copy
                    Some((hd, _)) => hd.clone(),
                    None => Node::List(vec![])
                }
            }
        }

        panic!("`car` takes only a quoted list, but got {:?}", args);
    }

    fn cdr(&self, args: &[Node], node: &Node) -> Node {
        if args.len() == 1 {
            if let &Node::QuotedList(ref xs) = &args[0] {
                return match xs.split_first() {
                    // TODO: Don't copy
                    Some((_, tl)) => Node::QuotedList(tl.to_vec()),
                    None => Node::List(vec![])
                }
            }
        }

        panic!("`car` takes only a quoted list, but got {:?}", args);
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
                            "car" => self.car(tl, &node),
                            "cdr" => self.cdr(tl, &node),
                            _ => panic!("Unexpected node: {:?}", node),
                        },
                    _ => panic!("Unexpected node: {:?}", node),
                }
            },
            Node::QuotedList(x) => Node::List(x),
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

        assert_eq!(
            Node::Integer(42),
            Eval::new().eval(
                Node::List(
                    vec![
                        Node::Keyword(String::from("car")),
                        Node::QuotedList(
                            vec![
                                Node::Integer(42),
                                Node::Integer(-123),
                                Node::Integer(0),
                            ]
                        ),
                    ]
                )
            )
        );

        assert_eq!(
            Node::QuotedList(
                vec![
                    Node::Integer(4),
                    Node::Integer(2),
                ]
            ),
            Eval::new().eval(
                Node::List(
                    vec![
                        Node::Keyword(String::from("cdr")),
                        Node::QuotedList(
                            vec![
                                Node::Integer(0),
                                Node::Integer(4),
                                Node::Integer(2),
                            ]
                        ),
                    ]
                )
            )
        );
    }
}

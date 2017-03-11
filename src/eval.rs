use std::collections::HashMap;
use parser::Node;

struct Eval;

impl Eval {
    pub fn new() -> Self {
        Eval {}
    }

    fn calc_integer<F>(&self, env: &mut HashMap<String, Node>, f: &F, args: &[Node], node: &Node) -> Node
        where F: Fn(i64, i64) -> i64 {

        Node::Integer(
            args.iter().fold(None, |a, x| match &self.eval(env, x.clone()) {
                &Node::Integer(i) => match a {
                    Some(aa) => Some(f(aa, i)),
                    None => Some(i)
                },
                _ => panic!("{:?} takes only an integer, but got {:?}", node, x)
            }
        ).unwrap_or(0))
    }

    fn car(&self, env: &mut HashMap<String, Node>, args: &[Node], node: &Node) -> Node {
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

    fn cdr(&self, env: &mut HashMap<String, Node>, args: &[Node], node: &Node) -> Node {
        if args.len() == 1 {
            if let &Node::QuotedList(ref xs) = &args[0] {
                return match xs.split_first() {
                    // TODO: Don't copy
                    Some((_, tl)) => Node::QuotedList(tl.to_vec()),
                    None => Node::List(vec![])
                }
            }
        }

        panic!("`cdr` takes only a quoted list, but got {:?}", args);
    }

    fn setq(&self, env: &mut HashMap<String, Node>, args: &[Node], node: &Node) -> Node {
        if args.len() % 2 == 0 {
            let mut key = None;
            for arg in args {
                if let Some(k) = key {
                    // TODO: Don't copy
                    env.insert(k, arg.clone());
                    key = None;
                }
                else {
                    if let &Node::Keyword(ref k) = arg {
                        // TODO: Don't copy
                        key = Some(k.clone())
                    }
                    else {
                        panic!("`setq` accepts only Node::Keyword as a key, but got {:?}", arg);
                    }
                }
            }
        }

        panic!("`setq` takes only key value pairs, but got {:?}", args);
    }

    pub fn eval(&self, env: &mut HashMap<String, Node>, node: Node) -> Node {
        match node {
            Node::Integer(_) => node,
            Node::Keyword(kwd) => {
                // TODO: Don't copy
                let cloned_env = env.clone();
                let x = cloned_env.get(&kwd).unwrap();
                self.eval(env, x.clone())
            }
            Node::List(ref xs) => {
                let (hd, tl) = xs.split_first().unwrap();
                match hd {
                    &Node::Keyword(ref kwd) => 
                        match kwd.as_str() {
                            "+" => self.calc_integer(env, &|a, i| a + i, tl, &node),
                            "-" => self.calc_integer(env, &|a, i| a - i, tl, &node),
                            "*" => self.calc_integer(env, &|a, i| a * i, tl, &node),
                            "/" => self.calc_integer(env, &|a, i| a / i, tl, &node),
                            "car" => self.car(env, tl, &node),
                            "cdr" => self.cdr(env, tl, &node),
                            "setq" => self.setq(env, tl, &node),
                            kwd => {
                                // TODO: Don't copy
                                let cloned_env = env.clone();
                                let x = cloned_env.get(kwd).unwrap();
                                self.eval(env, x.clone())
                            },
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
    use std::collections::HashMap;
    use super::Eval;
    use parser::Node;

    #[test]
    fn eval() {
        assert_eq!(
            Node::Integer(42),
            Eval::new().eval(
                &mut HashMap::new(),
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
                &mut HashMap::new(),
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
                &mut HashMap::new(),
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
                &mut HashMap::new(),
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

        let env = &mut HashMap::new();
        env.insert(String::from("x"), Node::Integer(40));
        assert_eq!(
            Node::Integer(42),
            Eval::new().eval(
                env,
                Node::List(
                    vec![
                        Node::Keyword(String::from("+")),
                        Node::Integer(2),
                        Node::Keyword(String::from("x"))
                    ]
                )
            )
        );
    }
}

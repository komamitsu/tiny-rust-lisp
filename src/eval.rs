use std::collections::HashMap;
use parser::Node;

pub struct Eval;

// TODO: Reduce memory copy...
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

    fn cond<F>(&self, env: &mut HashMap<String, Node>, f: &F, args: &[Node], node: &Node) -> Node
        where F: Fn(i64, i64) -> bool {

        let result = args.iter().fold(None, |a, x| match &self.eval(env, x.clone()) {
            &Node::Integer(i) => match a {
                Some((result, prev)) => Some((result && f(prev, i), i)),
                None => Some((true, i))
            },
            _ => panic!("{:?} takes only an integer, but got {:?}", node, x)
        }).unwrap();

        if result.0 {
            Node::True
        }
        else {
            Node::False
        }
    }

    fn if_then_else(&self, env: &mut HashMap<String, Node>, args: &[Node], node: &Node) -> Node {
        if args.len() >= 2 && args.len() < 4 {
            return match &self.eval(env, args[0].clone()) {
                &Node::True => self.eval(env, args[1].clone()),
                &Node::False => {
                    if args.len() == 3 {
                        self.eval(env, args[2].clone())
                    }
                    else {
                        Node::List(Vec::new())
                    }
                },
                _ => panic!("The 1st parameter of `if` should be boolean, but got {:?}", args)
            }
        }

        panic!("`if` takes 2 or 3 arguments, but got {:?}", args);
    }

    fn car(&self, env: &mut HashMap<String, Node>, args: &[Node], node: &Node) -> Node {
        if args.len() == 1 {
            if let &Node::QuotedList(ref xs) = &args[0] {
                return match xs.split_first() {
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
                    let evalated_node = self.eval(env, arg.clone());
                    env.insert(k, evalated_node);
                    key = None;
                }
                else {
                    if let &Node::Keyword(ref k) = arg {
                        key = Some(k.clone())
                    }
                    else {
                        panic!("`setq` accepts only Node::Keyword as a key, but got {:?}", arg);
                    }
                }
            }
            return node.clone();
        }

        panic!("`setq` takes only key value pairs, but got {:?}", args);
    }

    fn lambda(&self, env: &mut HashMap<String, Node>, args: &[Node], node: &Node) -> Node {
        if args.len() == 2 {
            match (&args[0], &args[1]) {
                (&Node::List(ref xs), &Node::List(ref body)) => return 
                        Node::Func(
                            xs.iter().map(
                                |x| match x {
                                    &Node::Keyword(ref kwd) => kwd.clone(),
                                    _ => panic!("The 2nd parameter of `lambda` should be a list of keywords, but got {:?}", args)
                                }).collect::<Vec<String>>(), body.clone()),
                _ => ()
            }
        }

        panic!("`lambda` takes only (name:keyword args:list body:list), but got {:?}", args);
    }

    fn call(&self, env: &mut HashMap<String, Node>, args: &[Node], node: &Node) -> Node {
        match node {
            &Node::Func(ref xs, ref body) => {
                let cloned_env = env.clone();
                let saved_env_kvs = xs.iter().map(|k| (k, cloned_env.get(k))).collect::<Vec<(&String, Option<&Node>)>>();
                for (i, x) in xs.iter().enumerate() {
                    env.insert(x.clone(), args[i].clone());
                }
                // Dynamic scope not lexical scope
                let result = self.eval(env, Node::List(body.clone()));
                for x in xs {
                    env.remove(x);
                }
                for (k, v) in saved_env_kvs {
                    if let Some(saved_node) = v {
                        env.insert(k.clone(), saved_node.clone());
                    }
                }
                return result;
            },
            _ => ()
        }

        panic!("Failed to call a function due to unexpected arguments: node is {:?} and arguments are {:?}", node, args);
    }

    pub fn eval(&self, env: &mut HashMap<String, Node>, node: Node) -> Node {
        match node {
            Node::Integer(_) => node,
            Node::Keyword(kwd) => {
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
                            "=" => self.cond(env, &|a, i| a == i, tl, &node),
                            ">" => self.cond(env, &|a, i| a > i, tl, &node),
                            ">=" => self.cond(env, &|a, i| a >= i, tl, &node),
                            "<" => self.cond(env, &|a, i| a < i, tl, &node),
                            "<=" => self.cond(env, &|a, i| a <= i, tl, &node),
                            "/=" => self.cond(env, &|a, i| a != i, tl, &node),
                            "if" => self.if_then_else(env, tl, &node),
                            "car" => self.car(env, tl, &node),
                            "cdr" => self.cdr(env, tl, &node),
                            "setq" => self.setq(env, tl, &node),
                            "lambda" => self.lambda(env, tl, &node),
                            _ =>  {
                                let cloned_env = env.clone();
                                match cloned_env.get(kwd) {
                                    Some(f) => self.call(env, tl, &f),
                                    None => panic!("Unknown keyword: {:?}", kwd)
                                }
                            }
                        },
                    _ => panic!("Unexpected node: {:?}", node),
                }
            },
            Node::QuotedList(x) => Node::List(x),
            _ => node,
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

        let env0 = &mut HashMap::new();
        env0.insert(String::from("x"), Node::Integer(40));
        assert_eq!(
            Node::Integer(42),
            Eval::new().eval(
                env0,
                Node::List(
                    vec![
                        Node::Keyword(String::from("+")),
                        Node::Integer(2),
                        Node::Keyword(String::from("x"))
                    ]
                )
            )
        );

        let env1 = &mut HashMap::new();
        Eval::new().eval(
            env1,
            Node::List(
                vec![
                    Node::Keyword(String::from("setq")),
                    Node::Keyword(String::from("add")),
                    Node::List(
                        vec![
                            Node::Keyword(String::from("lambda")),
                            Node::List(
                                vec![
                                    Node::Keyword(String::from("a")),
                                    Node::Keyword(String::from("b")),
                                ]
                            ),
                            Node::List(
                                vec![
                                    Node::Keyword(String::from("+")),
                                    Node::Keyword(String::from("a")),
                                    Node::Keyword(String::from("b")),
                                ]
                            ),
                        ]
                    )
                ]
            )
        );
        println!("add is {:?}", env1.get("add"));

        assert_eq!(
            Node::Integer(42),
            Eval::new().eval(
                env1,
                Node::List(
                    vec![
                        Node::Keyword(String::from("add")),
                        Node::Integer(40),
                        Node::Integer(2),
                    ]
                )
            )
        );
    }
}

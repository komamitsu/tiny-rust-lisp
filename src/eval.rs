use std::collections::HashMap;
use std::iter::Map;
use parser::Node;

#[derive(Debug, Clone)]
pub struct EvalError(String);

pub struct Eval;

// TODO: Reduce memory copy...
impl Eval {
    pub fn new() -> Self {
        Eval {}
    }

    fn calc_integer<F>(&self,
                       env: &mut HashMap<String, Node>,
                       f: &F,
                       args: &[Node], node: &Node) -> Result<Node, EvalError>
        where F: Fn(i64, i64) -> i64 {

        let result =
            args.iter().fold(
                None,
                |a, x| {
                    let nd = self.eval(env, x.clone());
                    match nd {
                        Ok(Node::Integer(i)) => match a {
                            Some(Ok(aa)) => Some(Ok(f(aa, i))),
                            Some(Err(err)) => Some(Err(err)),
                            None => Some(Ok(i)),
                        },
                        Ok(_) => Some(Err(EvalError(format!("{:?} takes only an integer, but got {:?}", node, x)))),
                        Err(err) => Some(Err(err)),
                    }
                }
            );

        match result {
            Some(Ok(i)) => Ok(Node::Integer(i)),
            Some(Err(err)) => Err(err),
            None => Err(EvalError(format!("Empty argument")))
        }
    }

    fn cond<F>(&self,
               env: &mut HashMap<String, Node>,
               f: &F,
               args: &[Node],
               node: &Node) -> Result<Node, EvalError>
        where F: Fn(i64, i64) -> bool {

        let result =
            args.iter().fold(
                None,
                |a, x| {
                    let nd = self.eval(env, x.clone());
                    match nd {
                        Ok(Node::Integer(i)) => match a {
                            Some(Ok((result, prev))) => Some(Ok((result && f(prev, i), i))),
                            Some(Err(err)) => Some(Err(err)),
                            None => Some(Ok((true, i))),
                        },
                        Ok(_) => Some(Err(EvalError(format!("{:?} takes only an integer, but got {:?}", node, x)))),
                        Err(err) => Some(Err(err)),
                    }
                }
            );

        match result {
            Some(Ok(result)) => Ok(if result.0 { Node::True } else { Node::False }),
            Some(Err(err)) => Err(err),
            None => Err(EvalError(format!("Empty argument")))
        }
    }

    fn if_then_else(&self,
                    env: &mut HashMap<String, Node>,
                    args: &[Node],
                    node: &Node) -> Result<Node, EvalError> {

        if args.len() < 2 || args.len() > 3 {
            return Err(EvalError(format!("`if` takes 2 or 3 arguments, but got {:?}", args)));
        }

        Ok(match &try!(self.eval(env, args[0].clone())) {
            &Node::True => try!(self.eval(env, args[1].clone())),
            &Node::False => {
                if args.len() == 3 {
                    try!(self.eval(env, args[2].clone()))
                }
                else {
                    Node::List(Vec::new())
                }
            },
            _ => return Err(EvalError(format!("The 1st parameter of `if` should be boolean, but got {:?}", args)))
        })
    }

    fn car(&self,
           env: &mut HashMap<String, Node>,
           args: &[Node],
           node: &Node) -> Result<Node, EvalError> {

        if args.len() == 1 {
            if let &Node::QuotedList(ref xs) = &args[0] {
                return Ok(match xs.split_first() {
                    Some((hd, _)) => hd.clone(),
                    None => Node::List(vec![])
                })
            }
        }

        Err(EvalError(format!("`car` takes only a quoted list, but got {:?}", args)))
    }

    fn cdr(&self,
           env: &mut HashMap<String, Node>,
           args: &[Node],
           node: &Node) -> Result<Node, EvalError> {

        if args.len() == 1 {
            if let &Node::QuotedList(ref xs) = &args[0] {
                return Ok(match xs.split_first() {
                    Some((_, tl)) => Node::QuotedList(tl.to_vec()),
                    None => Node::List(vec![])
                })
            }
        }

        Err(EvalError(format!("`cdr` takes only a quoted list, but got {:?}", args)))
    }

    fn setq(&self,
            env: &mut HashMap<String, Node>,
            args: &[Node],
            node: &Node) -> Result<Node, EvalError> {

        if args.len() % 2 == 0 {
            let mut key = None;
            for arg in args {
                if let Some(k) = key {
                    let evalated_node = try!(self.eval(env, arg.clone()));
                    env.insert(k, evalated_node);
                    key = None;
                }
                else {
                    if let &Node::Keyword(ref k) = arg {
                        key = Some(k.clone())
                    }
                    else {
                        return Err(EvalError(format!("`setq` accepts only Node::Keyword as a key, but got {:?}", arg)));
                    }
                }
            }
            return Ok(node.clone())
        }

        Err(EvalError(format!("`setq` takes only key value pairs, but got {:?}", args)))
    }

    fn lambda(&self, env: &mut HashMap<String, Node>, args: &[Node], node: &Node) -> Result<Node, EvalError> {
        if args.len() == 2 {
            match (&args[0], &args[1]) {
                (&Node::List(ref xs), &Node::List(ref body)) => {
                    let mut fargs = Vec::new();
                    let mut errors = Vec::new();

                    for x in xs {
                        match x {
                            &Node::Keyword(ref kwd) => fargs.push(kwd.clone()),
                            _ => errors.push(
                                EvalError(format!("The 2nd parameter of `lambda` should be a list of keywords, but got {:?}", args))
                                ),
                        }
                    }

                    if ! errors.is_empty() {
                        return Err(errors[0].clone())
                    }

                    return Ok(Node::Func(fargs, body.clone()))
                },
                _ => ()
            }
        }

        Err(EvalError(format!("`lambda` takes only (name:keyword args:list body:list), but got {:?}", args)))
    }

    fn call(&self, env: &mut HashMap<String, Node>, args: &[Node], node: &Node) -> Result<Node, EvalError> {
        match node {
            &Node::Func(ref xs, ref body) => {
                let cloned_env = env.clone();

                let saved_env_kvs = xs.iter().
                    map(|k| (k, cloned_env.get(k))).
                    collect::<Vec<(&String, Option<&Node>)>>();

                for (i, x) in xs.iter().enumerate() {
                    let evaled_arg = try!(self.eval(env, args[i].clone()));
                    env.insert(x.clone(), evaled_arg);
                }

                // Dynamic scope, not lexical scope
                let result = try!(self.eval(env, Node::List(body.clone())));
                for x in xs {
                    env.remove(x);
                }
                for (k, v) in saved_env_kvs {
                    if let Some(saved_node) = v {
                        env.insert(k.clone(), saved_node.clone());
                    }
                }
                return Ok(result)
            },
            _ => ()
        }

        Err(EvalError(format!("Failed to call a function due to unexpected arguments: node is {:?} and arguments are {:?}", node, args)))
    }

    pub fn eval(&self, env: &mut HashMap<String, Node>, node: Node) -> Result<Node, EvalError> {
        Ok(
            match node {
                Node::Integer(_) => node,
                Node::Keyword(kwd) => {
                    let cloned_env = env.clone();
                    match cloned_env.get(&kwd) {
                        Some(x) => try!(self.eval(env, x.clone())),
                        None => Node::Keyword(kwd),
                    }
                },
                Node::List(ref xs) => {
                    let (hd, tl) = xs.split_first().unwrap();
                    match hd {
                        &Node::Keyword(ref kwd) => {
                            match kwd.as_str() {
                                "+" => try!(self.calc_integer(env, &|a, i| a + i, tl, &node)),
                                "-" => try!(self.calc_integer(env, &|a, i| a - i, tl, &node)),
                                "*" => try!(self.calc_integer(env, &|a, i| a * i, tl, &node)),
                                "/" => try!(self.calc_integer(env, &|a, i| a / i, tl, &node)),
                                "=" => try!(self.cond(env, &|a, i| a == i, tl, &node)),
                                ">" => try!(self.cond(env, &|a, i| a > i, tl, &node)),
                                ">=" => try!(self.cond(env, &|a, i| a >= i, tl, &node)),
                                "<" => try!(self.cond(env, &|a, i| a < i, tl, &node)),
                                "<=" => try!(self.cond(env, &|a, i| a <= i, tl, &node)),
                                "/=" => try!(self.cond(env, &|a, i| a != i, tl, &node)),
                                "if" => try!(self.if_then_else(env, tl, &node)),
                                "car" => try!(self.car(env, tl, &node)),
                                "cdr" => try!(self.cdr(env, tl, &node)),
                                "setq" => try!(self.setq(env, tl, &node)),
                                "lambda" => try!(self.lambda(env, tl, &node)),
                                _ =>  {
                                    let cloned_env = env.clone();
                                    match cloned_env.get(kwd) {
                                        Some(f) => try!(self.call(env, &tl, &f)),
                                        None => return Err(EvalError(format!("Unknown keyword: {:?}", kwd)))
                                    }
                                }
                            }
                        },
                        _ => return Err(EvalError(format!("Unexpected node: {:?}", node))),
                    }
                },
                Node::QuotedList(x) => Node::List(x),
                _ => node,
            }
        )
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
            ).unwrap()
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
            ).unwrap()
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
            ).unwrap()
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
            ).unwrap()
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
            ).unwrap()
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
            ).unwrap()
        );
    }
}


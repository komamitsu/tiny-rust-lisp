use std::collections::HashMap;
use std::rc::Rc;
use parser::Node;

#[derive(Debug, Clone)]
pub struct EvalError(String);

pub struct Eval;

#[derive(Debug, Clone)]
pub struct Env {
    envs: Vec<HashMap<String, Rc<Node>>>
}

impl Env {
    pub fn new() -> Self {
        Env {
            envs: vec![HashMap::new()]
        }
    }

    pub fn new_with_map(map: HashMap<String, Node>) -> Self {
        Env {
            envs: vec![map.iter().map(|(k, v)| (k.clone(), Rc::new(v.clone()))).collect::<HashMap<String, Rc<Node>>>()]
        }
    }

    pub fn get(&self, key: &String) -> Option<Rc<Node>> {
        // for env in &self.envs {
        for i in 0..self.envs.len() {
            let env = &self.envs[self.envs.len() - 1 - i];
            if let Some(v) = env.get(key) {
                return Some(v.clone())
            }
        }
        None
    }

    pub fn insert(&mut self, k: String, v: Rc<Node>) -> Option<Rc<Node>> {
        if self.envs.is_empty() {
            panic!("Env#insert shouldn't be called for empty `envs`");
        }
        let len = self.envs.len();
        let env = &mut self.envs[len - 1];
        env.insert(k, v)
    }

    pub fn remove(&mut self, k: &String) -> Option<Rc<Node>> {
        if self.envs.is_empty() {
            panic!("Env#remove shouldn't be called for empty `envs`");
        }
        let len = self.envs.len();
        let env = &mut self.envs[len - 1];
        env.remove(k)
    }

    pub fn push_env(&mut self) {
        self.envs.push(HashMap::new());
    }

    pub fn pop_env(&mut self) {
        self.envs.pop();
    }
}

// TODO: Reduce memory copy...
impl Eval {
    pub fn new() -> Self {
        Eval {}
    }

    fn calc_integer<F>(&self,
                       env: &mut Env,
                       f: &F,
                       args: &[Rc<Node>], node: &Rc<Node>) -> Result<Rc<Node>, EvalError>
        where F: Fn(i64, i64) -> i64 {

        let result =
            args.iter().fold(
                None,
                |a, x| {
                    let nd = self.eval(env, x.clone());
                    match nd {
                        Ok(rcnode) => match *rcnode.clone() {
                            Node::Integer(i) => match a {
                                Some(Ok(aa)) => Some(Ok(f(aa, i))),
                                Some(Err(err)) => Some(Err(err)),
                                None => Some(Ok(i)),
                            },
                            _ => Some(Err(EvalError(
                                        format!("{:?} takes only an integer, but got {:?}", node, x)))),
                        },
                        Err(err) => Some(Err(err)),
                    }
                }
            );

        match result {
            Some(Ok(i)) => Ok(Rc::new(Node::Integer(i))),
            Some(Err(err)) => Err(err),
            None => Err(EvalError(format!("Empty argument")))
        }
    }

    fn cond<F>(&self,
               env: &mut Env,
               f: &F,
               args: &[Rc<Node>],
               node: &Rc<Node>) -> Result<Rc<Node>, EvalError>
        where F: Fn(i64, i64) -> bool {

        let result =
            args.iter().fold(
                None,
                |a, x| {
                    let nd = self.eval(env, x.clone());
                    match nd {
                        Ok(rcnode) => match *rcnode.clone() {
                            Node::Integer(i) => match a {
                                Some(Ok((result, prev))) => Some(Ok((result && f(prev, i), i))),
                                Some(Err(err)) => Some(Err(err)),
                                None => Some(Ok((true, i))),
                            },
                            _ => Some(Err(EvalError(
                                        format!("{:?} takes only an integer, but got {:?}", node, x)))),
                        },
                        Err(err) => Some(Err(err)),
                    }
                }
            );

        match result {
            Some(Ok(result)) => Ok(if result.0 { Rc::new(Node::True) }
                                   else { Rc::new(Node::False) }),
            Some(Err(err)) => Err(err),
            None => Err(EvalError(format!("Empty argument")))
        }
    }

    fn if_then_else(&self,
                    env: &mut Env,
                    args: &[Rc<Node>],
                    node: &Rc<Node>) -> Result<Rc<Node>, EvalError> {

        if args.len() < 2 || args.len() > 3 {
            return Err(EvalError(
                    format!("`if` takes 2 or 3 arguments, but got {:?}", args)));
        }

        Ok(match *try!(self.eval(env, args[0].clone())).clone() {
            Node::True => try!(self.eval(env, args[1].clone())),
            Node::False => {
                if args.len() == 3 {
                    try!(self.eval(env, args[2].clone()))
                }
                else {
                    Rc::new(Node::List(Vec::new()))
                }
            },
            _ => return Err(EvalError(
                    format!("The 1st parameter of `if` should be boolean, but got {:?}", args)))
        })
    }

    fn car(&self,
           env: &mut Env,
           args: &[Rc<Node>],
           node: &Rc<Node>) -> Result<Rc<Node>, EvalError> {

        if args.len() == 1 {
            if let Node::QuotedList(ref xs) = *args[0].clone() {
                return Ok(match xs.split_first() {
                    Some((hd, _)) => hd.clone(),
                    None => Rc::new(Node::List(Vec::new()))
                })
            }
        }

        Err(EvalError(format!("`car` takes only a quoted list, but got {:?}", args)))
    }

    fn cdr(&self,
           env: &mut Env,
           args: &[Rc<Node>],
           node: &Rc<Node>) -> Result<Rc<Node>, EvalError> {

        if args.len() == 1 {
            if let Node::QuotedList(ref xs) = *args[0].clone() {
                return Ok(
                    match xs.split_first() {
                        Some((_, tl)) => Rc::new(Node::QuotedList((*tl.clone()).to_vec())),
                        None => Rc::new(Node::List(Vec::new()))
                    }
                )
            }
        }

        Err(EvalError(format!("`cdr` takes only a quoted list, but got {:?}", args)))
    }

    fn setq(&self,
            env: &mut Env,
            args: &[Rc<Node>],
            node: &Rc<Node>) -> Result<Rc<Node>, EvalError> {

        if args.len() % 2 == 0 {
            let mut key = None;
            for arg in args {
                if let Some(k) = key {
                    let evalated_node = try!(self.eval(env, arg.clone()));
                    env.insert(k, evalated_node);
                    key = None;
                }
                else {
                    if let Node::Keyword(ref k) = *arg.clone() {
                        key = Some(k.clone())
                    }
                    else {
                        return Err(EvalError(format!(
                                    "`setq` accepts only Node::Keyword as a key, but got {:?}", arg)));
                    }
                }
            }
            return Ok(node.clone())
        }

        Err(EvalError(
                format!("`setq` takes only key value pairs, but got {:?}", args)))
    }

    fn lambda(&self, env: &mut Env, args: &[Rc<Node>], node: &Rc<Node>) -> Result<Rc<Node>, EvalError> {
        if args.len() == 2 {
            match (&*args[0].clone(), &*args[1].clone()) {
                (&Node::List(ref xs), &Node::List(ref body)) => {
                    let mut fargs = Vec::new();
                    let mut errors = Vec::new();

                    for x in xs {
                        match *x.clone() {
                            Node::Keyword(ref kwd) => fargs.push(kwd.clone()),
                            _ => errors.push(
                                    EvalError(format!(
                                        "The 2nd parameter of `lambda` should be a list of keywords, but got {:?}", args))
                                ),
                        }
                    }

                    if ! errors.is_empty() {
                        return Err(errors[0].clone())
                    }

                    // TODO: Avoid copying
                    return Ok(Rc::new(Node::Func(fargs, body.clone())))
                },
                _ => ()
            }
        }

        Err(EvalError(
                format!("`lambda` takes only (name:keyword args:list body:list), but got {:?}", args)))
    }

    fn call(&self, env: &mut Env, args: &[Rc<Node>], node: &Rc<Node>) -> Result<Rc<Node>, EvalError> {
        match *node.clone() {
            Node::Func(ref xs, ref body) => {
                if xs.len() != args.len() {
                    panic!("The numbers of argments don't match: expected({:?}), got({:?})", xs, args);
                }

                env.push_env();

                for (i, x) in xs.iter().enumerate() {
                    let evaled_arg = try!(self.eval(env, args[i].clone()));
                    env.insert(x.clone(), evaled_arg);
                }

                let result = try!(self.eval(env, Rc::new(Node::List(body.clone()))));

                env.pop_env();

                return Ok(result)
            },
            _ => ()
        }

        Err(EvalError(
                format!("Failed to call a function due to unexpected arguments: node is {:?} and arguments are {:?}",
                        node, args)))
    }

    pub fn eval(&self, env: &mut Env, node: Rc<Node>) -> Result<Rc<Node>, EvalError> {
        match &*node.clone() {
            &Node::Integer(_) => Ok(node),
            &Node::Keyword(ref kwd) => {
                let nd = match env.get(&kwd) {
                    Some(x) => x,
                    None => return Ok(Rc::new(Node::Keyword(kwd.clone()))),
                };
                self.eval(env, nd.clone())
            },
            &Node::List(ref xs) => self.eval_func(env, &node, xs),
            // TODO: Avoid copying
            &Node::QuotedList(ref x) => Ok(Rc::new(Node::List(x.clone()))),
            _ => Ok(node),
        }
    }

    fn eval_func(&self, env: &mut Env, node: &Rc<Node>, xs: &Vec<Rc<Node>>) -> Result<Rc<Node>, EvalError> {
        let (hd, tl) = xs.split_first().unwrap();
        if let &Node::Keyword(ref kwd) = &*hd.clone() {
            match kwd.as_str() {
                "+" => self.calc_integer(env, &|a, i| a + i, tl, node),
                "-" => self.calc_integer(env, &|a, i| a - i, tl, node),
                "*" => self.calc_integer(env, &|a, i| a * i, tl, node),
                "/" => self.calc_integer(env, &|a, i| a / i, tl, node),
                "=" => self.cond(env, &|a, i| a == i, tl, node),
                ">" => self.cond(env, &|a, i| a > i, tl, node),
                ">=" => self.cond(env, &|a, i| a >= i, tl, node),
                "<" => self.cond(env, &|a, i| a < i, tl, node),
                "<=" => self.cond(env, &|a, i| a <= i, tl, node),
                "/=" => self.cond(env, &|a, i| a != i, tl, node),
                "if" => self.if_then_else(env, tl, node),
                "car" => self.car(env, tl, node),
                "cdr" => self.cdr(env, tl, node),
                "setq" => self.setq(env, tl, node),
                "lambda" => self.lambda(env, tl, node),
                _ =>  {
                    let f = match env.get(kwd) {
                        Some(f) => f.clone(),
                        None => return Err(EvalError(format!("Unknown keyword: {:?}", kwd)))
                    };
                    self.call(env, &tl, &f)
                }
            }
        }
        else {
            Err(EvalError(format!("Unexpected node: {:?}", node)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval() {
        assert_eq!(
            Node::Integer(42),
            *Eval::new().eval(
                &mut Env::new(),
                Rc::new(Node::List(
                    vec![
                        Rc::new(Node::Keyword(String::from("+"))),
                        Rc::new(Node::Integer(2)),
                        Rc::new(Node::Integer(40)),
                    ]
                ))
            ).unwrap()
        );

        assert_eq!(
            Node::Integer(42),
            *Eval::new().eval(
                &mut Env::new(),
                Rc::new(Node::List(
                    vec![
                        Rc::new(Node::Keyword(String::from("*"))),
                        Rc::new(Node::Integer(6)),
                        Rc::new(Node::List(
                            vec![
                                Rc::new(Node::Keyword(String::from("-"))),
                                Rc::new(Node::Integer(42)),
                                Rc::new(Node::Integer(35)),
                            ]
                        )),
                    ]
                ))
            ).unwrap()
        );

        assert_eq!(
            Node::Integer(42),
            *Eval::new().eval(
                &mut Env::new(),
                Rc::new(Node::List(
                    vec![
                        Rc::new(Node::Keyword(String::from("car"))),
                        Rc::new(Node::QuotedList(
                            vec![
                                Rc::new(Node::Integer(42)),
                                Rc::new(Node::Integer(-123)),
                                Rc::new(Node::Integer(0)),
                            ]
                        )),
                    ]
                ))
            ).unwrap()
        );

        assert_eq!(
            Node::QuotedList(
                vec![
                    Rc::new(Node::Integer(4)),
                    Rc::new(Node::Integer(2)),
                ]
            ),
            *Eval::new().eval(
                &mut Env::new(),
                Rc::new(Node::List(
                    vec![
                        Rc::new(Node::Keyword(String::from("cdr"))),
                        Rc::new(Node::QuotedList(
                            vec![
                                Rc::new(Node::Integer(0)),
                                Rc::new(Node::Integer(4)),
                                Rc::new(Node::Integer(2)),
                            ]
                        )),
                    ]
                ))
            ).unwrap()
        );

        let env0 = &mut Env::new();
        env0.insert(String::from("x"), Rc::new(Node::Integer(40)));
        assert_eq!(
            Node::Integer(42),
            *Eval::new().eval(
                env0,
                Rc::new(Node::List(
                    vec![
                        Rc::new(Node::Keyword(String::from("+"))),
                        Rc::new(Node::Integer(2)),
                        Rc::new(Node::Keyword(String::from("x")))
                    ]
                ))
            ).unwrap()
        );

        let env1 = &mut Env::new();
        Eval::new().eval(
            env1,
            Rc::new(Node::List(
                vec![
                    Rc::new(Node::Keyword(String::from("setq"))),
                    Rc::new(Node::Keyword(String::from("add"))),
                    Rc::new(Node::List(
                        vec![
                            Rc::new(Node::Keyword(String::from("lambda"))),
                            Rc::new(Node::List(
                                vec![
                                    Rc::new(Node::Keyword(String::from("a"))),
                                    Rc::new(Node::Keyword(String::from("b"))),
                                ]
                            )),
                            Rc::new(Node::List(
                                vec![
                                    Rc::new(Node::Keyword(String::from("+"))),
                                    Rc::new(Node::Keyword(String::from("a"))),
                                    Rc::new(Node::Keyword(String::from("b"))),
                                ]
                            )),
                        ]
                    ))
                ]
            ))
        );

        assert_eq!(
            Node::Integer(42),
            *Eval::new().eval(
                env1,
                Rc::new(Node::List(
                    vec![
                        Rc::new(Node::Keyword(String::from("add"))),
                        Rc::new(Node::Integer(40)),
                        Rc::new(Node::Integer(2)),
                    ]
                ))
            ).unwrap()
        );
    }
}

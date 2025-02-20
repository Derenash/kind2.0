use crate::{*};

use std::collections::BTreeMap;
use std::collections::BTreeSet;

impl<'i> KindParser<'i> {

  // Parses a top-level use-declaration
  fn parse_use(&mut self, fid: u64, nam: String, uses: &mut Uses) -> Result<(), String> {
    self.skip_trivia();
    let add = self.parse_name()?;
    self.skip_trivia();
    let nam = format!("{}{}", nam, add);
    if self.peek_one() == Some('{') {
      self.consume("{")?;
      self.skip_trivia();
      loop {
        self.parse_use(fid, nam.clone(), uses)?;
        self.skip_trivia();
        match self.peek_one() {
          Some(',') => { self.consume(",")?; continue; }
          Some('}') => { self.consume("}")?; break; }
          _         => { return self.expected("comma (`,`) or semicolon (`;`)"); }
        }
      }
    } else {
      let pts = nam.split('.').collect::<Vec<&str>>();
      let key = pts.last().unwrap().to_string();
      //println!("use {} ~> {}", key, nam);
      uses.insert(key, nam);
    }
    return Ok(());
  }

  // Parses many top-level use-declarations
  fn parse_uses(&mut self, fid: u64) -> Result<Uses, String> {
    let mut uses = im::HashMap::new();
    self.skip_trivia();
    while self.peek_many(4) == Some("use ") {
      self.consume("use")?;
      self.parse_use(fid, String::new(), &mut uses)?;
      self.skip_trivia();
    }
    return Ok(uses);
  }

  // Parses a top-level definition (datatype or term)
  pub fn parse_def(&mut self, fid: u64, uses: &Uses) -> Result<(String, Term), String> {
    // Top-level datatype
    if self.peek_many(5) == Some("data ") {
      let adt = self.parse_adt(fid, uses)?;
      let mut typ = Term::Set;
      let mut val = Term::new_adt(&adt);
      for (idx_nam, idx_typ) in adt.idxs.iter().rev() {
        typ = Term::All { nam: idx_nam.clone(), inp: Box::new(idx_typ.clone()), bod: Box::new(typ) };
        val = Term::Lam { nam: idx_nam.clone(), bod: Box::new(val) };
      }
      for par_nam in adt.pars.iter().rev() {
        typ = Term::All { nam: par_nam.clone(), inp: Box::new(Term::Set), bod: Box::new(typ) };
        val = Term::Lam { nam: par_nam.clone(), bod: Box::new(val) };
      }
      //println!("NAM: {}", adt.name);
      //println!("TYP: {}", typ.show());
      //println!("VAL: {}", val.show());
      return Ok((adt.name, Term::Ann { chk: false, val: Box::new(val), typ: Box::new(typ) }));
    }

    // Top level definition
    self.skip_trivia();
    let nam = self.parse_name()?;
    let nam = uses.get(&nam).unwrap_or(&nam).to_string();
    self.skip_trivia();

    // Untyped Arguments (optional)
    let mut args = im::Vector::new();
    while self.peek_one().map_or(false, |c| c.is_ascii_alphabetic()) {
      let par = self.parse_name()?;
      self.skip_trivia();
      args.push_back((par, Term::Met{}));
    }

    // Typed Arguments (optional)
    while self.peek_one() == Some('(') {
      self.consume("(")?;
      let arg_name = self.parse_name()?;
      self.consume(":")?;
      let arg_type = self.parse_term(fid, uses)?;
      self.consume(")")?;
      args.push_back((arg_name, arg_type));
      self.skip_trivia();
    }

    // Type annotation (optional)
    let mut typ;
    let ann;
    if self.peek_one() == Some(':') {
      self.consume(":")?;
      typ = self.parse_term(fid, uses)?;
      ann = true;
    } else {
      typ = Term::Met {};
      ann = false;
    }

    // Value (mandatory)
    self.consume("=")?;
    let mut val = self.parse_term(fid, uses)?;

    // Adds lambdas/foralls for each argument
    typ.add_alls(args.clone());
    val.add_lams(args.clone());

    // Removes syntax-sugars from the AST
    typ.desugar();
    val.desugar();

    //println!("{}", nam);
    //println!(": {}", typ.show());
    //println!("= {}", val.show());
    //println!("");

    return Ok((nam, Term::Ann { chk: !ann, val: Box::new(val), typ: Box::new(typ) }));
  }

  // Parses a whole file into a book.
  pub fn parse_book(&mut self, name: &str, fid: u64) -> Result<Book, String> {
    let mut book = Book::new();
    // Parses all top-level 'use' declarations.
    let mut uses = self.parse_uses(fid)?;
    // Each file has an implicit 'use Path.to.file'. We add it here:
    uses.insert(name.split('.').last().unwrap().to_string(), name.to_string());
    // Parses all definitions.
    while *self.index() < self.input().len() {
      let (name, term) = self.parse_def(fid, &mut uses)?;
      book.defs.insert(name, term);
      self.skip_trivia();
    }
    Ok(book)
  }

}

impl Term {

  // Wraps a Lam around this term.
  fn add_lam(&mut self, arg: &str) {
    *self = Term::Lam {
      nam: arg.to_string(),
      bod: Box::new(std::mem::replace(self, Term::Met {})),
    };
  }

  // Wraps many lams around this term. Linearizes when possible.
  fn add_lams(&mut self, args: im::Vector<(String,Term)>) {
    // Passes through Src
    if let Term::Src { val, .. } = self {
      val.add_lams(args);
      return;
    }
    // Passes through Ann
    if let Term::Ann { val, .. } = self {
      val.add_lams(args);
      return;
    }
    // Linearizes a numeric pattern match
    if let Term::Swi { nam, z, s, .. } = self {
      if args.len() >= 1 && args[0].0 == *nam {
        let (head, tail) = args.split_at(1);
        z.add_lams(tail.clone());
        s.add_lams(tail.clone());
        self.add_lam(&head[0].0);
        return;
      }
    }
    // Linearizes a user-defined ADT match
    if let Term::Mch { mch } = self {
      let Match { name, cses, .. } = &mut **mch;
      if args.len() >= 1 && args[0].0 == *name {
        let (head, tail) = args.split_at(1);
        for (_, cse) in cses {
          cse.add_lams(tail.clone());
        }
        self.add_lam(&head[0].0);
        return;
      }
    }
    // Prepend remaining lambdas
    for (nam, _) in args.iter().rev() {
      self.add_lam(&nam);
    }
  }

  // Wraps an All around this term.
  fn add_all(&mut self, arg: &str, typ: &Term) {
    *self = Term::All {
      nam: arg.to_string(),
      inp: Box::new(typ.clone()),
      bod: Box::new(std::mem::replace(self, Term::Met {})),
    };
  }

  // Wraps many Lams around this term.
  fn add_alls(&mut self, args: im::Vector<(String,Term)>) {
    for (nam, typ) in args.iter().rev() {
      self.add_all(&nam, typ);
    }
  }

}

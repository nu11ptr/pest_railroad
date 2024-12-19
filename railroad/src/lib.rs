use std::mem;

use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;
use railroad::{
    Choice, Comment, Diagram, Empty, Node, NonTerminal, Optional, Repeat, Sequence, SimpleEnd,
    SimpleStart, Terminal, VerticalGrid,
};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PestParser;

fn make_expr(pairs: Pairs<Rule>) -> Box<dyn Node> {
    // Rule choices (or those without a choice operator this will be a single element)
    let mut choices: Vec<Vec<Box<dyn Node>>> = Vec::new();
    // Current choice
    let mut curr_choice: Vec<Box<dyn Node>> = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::term => {
                let term_pairs = pair.into_inner();

                // We might have a postfix operator, so store the term until we are sure
                let mut term: Option<Box<dyn Node>> = None;

                for term_pair in term_pairs {
                    match term_pair.as_rule() {
                        Rule::identifier => {
                            term = Some(Box::new(NonTerminal::new(term_pair.as_str().into())));
                        }
                        Rule::string => {
                            term = Some(Box::new(Terminal::new(term_pair.as_str().into())));
                        }
                        Rule::opening_paren | Rule::closing_paren => {
                            // No op - nothing to do
                        }
                        Rule::expression => {
                            term = Some(make_expr(term_pair.into_inner()));
                        }
                        Rule::repeat_operator => {
                            // Panic safety: Grammar ensures this must be populated by this point
                            term = Some(Box::new(Repeat::new(Empty, term.expect("term"))));
                        }
                        Rule::repeat_once_operator => {
                            // Panic safety: Grammar ensures this must be populated by this point
                            term = Some(Box::new(Repeat::new(term.expect("term"), Empty)));
                        }
                        Rule::optional_operator => {
                            // Panic safety: Grammar ensures this must be populated by this point
                            term = Some(Box::new(Optional::new(term.expect("term"))));
                        }
                        _ => {
                            // TODO: Replace with logging? (or perhaps store messages and return?)
                            eprintln!(
                                "### Unsupported rule in term: {:#?} ###",
                                term_pair.as_rule()
                            );
                        }
                    }
                }

                // Panic safety: Grammar ensures this must be populated by this point
                curr_choice.push(term.expect("store term"));
            }
            Rule::sequence_operator => {
                // No op - nothing to do
            }
            Rule::choice_operator => {
                // Store the current sequence and start a new one
                choices.push(mem::take(&mut curr_choice))
            }
            rule => unreachable!("Unexpected rule in expression: {rule:?}"),
        }
    }

    // Ensure that the last sequence is stored
    if !curr_choice.is_empty() {
        choices.push(curr_choice);
    }

    // Perform a custom flatten of our choices
    let mut choices: Vec<_> = choices
        .into_iter()
        .map(|mut seq| {
            match seq.len() {
                // This can only happpen if rule starts with a choice operator
                // TODO: Is empty the right choice here? What is the actual behavior of starting with a choice operator?
                0 => Box::new(Empty),
                // If we only have one element, return it directly
                1 => seq.remove(0),
                // Otherwise wrap in a sequence
                _ => Box::new(Sequence::new(seq)),
            }
        })
        .collect();

    // If we only have one choice, return it directly
    if choices.len() == 1 {
        choices.remove(0)
    } else {
        Box::new(Choice::new(choices))
    }
}

fn make_rule(identifier: &str, pairs: Pairs<Rule>) -> Box<dyn Node> {
    // Identifier stacked on top of a sequence
    let mut grid: Vec<Box<dyn Node>> = Vec::with_capacity(2);
    // Our rule sequence
    let mut seq: Vec<Box<dyn Node>> = Vec::with_capacity(pairs.len());

    let mut rule_ident = String::with_capacity(64);
    rule_ident.push_str(identifier);

    for pair in pairs {
        match pair.as_rule() {
            Rule::assignment_operator => {
                // No op - nothing to do
            }
            Rule::silent_modifier => {
                rule_ident.push_str(" (silent)");
            }
            Rule::atomic_modifier => {
                rule_ident.push_str(" (atomic)");
            }
            Rule::compound_atomic_modifier => {
                rule_ident.push_str(" (compound atomic)");
            }
            Rule::non_atomic_modifier => {
                rule_ident.push_str(" (non-atomic)");
            }
            Rule::opening_brace => {
                grid.push(Box::new(Comment::new(mem::take(&mut rule_ident))));
                seq.push(Box::new(SimpleStart));
            }
            Rule::expression => {
                seq.push(make_expr(pair.into_inner()));
            }
            Rule::closing_brace => {
                seq.push(Box::new(SimpleEnd));
            }
            rule => unreachable!("Unexpected rule in grammar rule: {rule:?}"),
        }
    }

    grid.push(Box::new(Sequence::new(seq)));
    Box::new(VerticalGrid::new(grid))
}

pub fn generate_diagram(
    input: &str,
) -> Result<Diagram<VerticalGrid<Box<dyn Node>>>, pest::error::Error<Rule>> {
    let pairs = PestParser::parse(Rule::grammar_rules, input)?;

    let mut nodes: Vec<Box<dyn Node>> = Vec::with_capacity(pairs.len());

    // Loop over all top level elements
    for pair in pairs {
        match pair.as_rule() {
            // We only process grammar rules
            Rule::grammar_rule => {
                let mut rule_pairs = pair.into_inner();

                // Panic safety: We know that the first element is either a line doc or an identifier from grammar
                let first_pair = rule_pairs.next().expect("line doc or identifier");

                match first_pair.as_rule() {
                    Rule::line_doc => {
                        nodes.push(Box::new(Comment::new(first_pair.as_str().into())));
                    }
                    Rule::identifier => {
                        nodes.push(make_rule(first_pair.as_str(), rule_pairs));
                    }
                    rule => unreachable!("Unexpected first rule in grammar rule: {rule:?}"),
                }
            }
            Rule::grammar_doc => {
                // No op - unsupported
            }
            Rule::EOI => {
                // No op - nothing to do
            }
            rule => unreachable!("Unexpected rule in top level grammar: {rule:?}"),
        }
    }

    let root = VerticalGrid::new(nodes);
    let diagram = Diagram::with_default_css(root);
    Ok(diagram)
}

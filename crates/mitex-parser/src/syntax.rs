//! Syntax kinds and typed syntax nodes

use mitex_lexer::{BraceKind, CommandName, Token};
use rowan::ast::AstNode;

macro_rules! from_repr {
	(
        #[repr($enum_repr:ty)]
		$(#[$meta:meta])*
		$vis:vis enum $name:ident {
			$($(#[$variant_meta:meta])* $variant:ident,)*
		}
	) => {
		$(#[$meta])*
        #[repr($enum_repr)]
		$vis enum $name {
			$($(#[$variant_meta])* $variant,)*
		}

		impl TryFrom<$enum_repr> for $name {
            type Error = ();

            /// Get the variant from a raw repr value
             fn try_from(kind: $enum_repr) -> Result<Self, ()> {
                const ALL: &'static [$name] = &[$($name::$variant,)*];

                ALL.get(usize::from(kind)).copied().ok_or(())
            }
		}
	};
}

from_repr! {
    #[repr(u16)]
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
    #[allow(missing_docs)]
    pub enum SyntaxKind {
        // Tokens
        TokenError,
        TokenLineBreak,
        TokenWhiteSpace,
        TokenComment,
        TokenLBrace,
        TokenRBrace,
        TokenLBracket,
        TokenRBracket,
        TokenLParen,
        TokenRParen,
        TokenComma,
        TokenTilde,
        TokenSlash,
        TokenWord,
        TokenDollar,
        TokenAmpersand,
        TokenHash,
        TokenUnderscore,
        TokenCaret,
        TokenApostrophe,
        TokenDitto,
        TokenSemicolon,
        TokenCommandSym,

        // Clauses
        ClauseCommandName,
        ClauseArgument,
        ClauseLR,

        // Items
        ItemNewLine,
        ItemText,
        ItemCurly,
        ItemBracket,
        ItemParen,
        ItemCmd,
        ItemEnv,
        ItemLR,
        ItemBegin,
        ItemEnd,
        ItemBlockComment,
        ItemTypstCode,
        ItemAttachComponent,
        ItemFormula,

        // Scopes
        ScopeRoot,
    }
}

impl From<Token> for SyntaxKind {
    fn from(kind: Token) -> Self {
        match kind {
            Token::LineBreak => SyntaxKind::TokenLineBreak,
            Token::Whitespace => SyntaxKind::TokenWhiteSpace,
            Token::LineComment => SyntaxKind::TokenComment,
            Token::Left(BraceKind::Curly) => SyntaxKind::TokenLBrace,
            Token::Right(BraceKind::Curly) => SyntaxKind::TokenRBrace,
            Token::Left(BraceKind::Bracket) => SyntaxKind::TokenLBracket,
            Token::Right(BraceKind::Bracket) => SyntaxKind::TokenRBracket,
            Token::Left(BraceKind::Paren) => SyntaxKind::TokenLParen,
            Token::Right(BraceKind::Paren) => SyntaxKind::TokenRParen,
            Token::Comma => SyntaxKind::TokenComma,
            Token::Tilde => SyntaxKind::TokenTilde,
            Token::Slash => SyntaxKind::TokenSlash,
            Token::Underscore => SyntaxKind::TokenUnderscore,
            Token::Apostrophe => SyntaxKind::TokenApostrophe,
            Token::Ditto => SyntaxKind::TokenDitto,
            Token::Semicolon => SyntaxKind::TokenSemicolon,
            Token::Caret => SyntaxKind::TokenCaret,
            Token::Word => SyntaxKind::TokenWord,
            Token::Dollar => SyntaxKind::TokenDollar,
            Token::Ampersand => SyntaxKind::TokenAmpersand,
            Token::Hash => SyntaxKind::TokenHash,
            Token::NewLine => SyntaxKind::ItemNewLine,
            Token::MacroArg(_) => SyntaxKind::TokenWord,
            Token::CommandName(
                CommandName::ErrorBeginEnvironment | CommandName::ErrorEndEnvironment,
            )
            | Token::Error => SyntaxKind::TokenError,
            Token::CommandName(CommandName::BeginEnvironment | CommandName::EndEnvironment) => {
                SyntaxKind::TokenCommandSym
            }
            Token::CommandName(_) => SyntaxKind::ClauseCommandName,
        }
    }
}

impl SyntaxKind {
    /// Checks whether the syntax kind is trivia
    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            SyntaxKind::TokenLineBreak
                | SyntaxKind::TokenWhiteSpace
                | SyntaxKind::TokenComment
                | SyntaxKind::ItemNewLine
        )
    }
}

use SyntaxKind::*;

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl From<rowan::SyntaxKind> for SyntaxKind {
    fn from(kind: rowan::SyntaxKind) -> Self {
        Self::try_from(kind.0).unwrap()
    }
}

/// Provides a TeX language for rowan
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TexLang {}

impl rowan::Language for TexLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        raw.into()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

/// exported tex syntax node
pub type SyntaxNode = rowan::SyntaxNode<TexLang>;
/// exported tex syntax token
pub type SyntaxToken = rowan::SyntaxToken<TexLang>;
/// exported tex syntax element
pub type SyntaxElement = rowan::SyntaxElement<TexLang>;

macro_rules! syntax_tree_node {
    ($(#[$attr:meta])* $name:ident, $($kind:pat),+) => {
        #[derive(Clone)]
        #[repr(transparent)]
        $(#[$attr])*
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            type Language = TexLang;

            fn can_cast(kind: SyntaxKind) -> bool {
                match kind {
                    $($kind => true,)+
                    _ => false,
                }
            }

            fn cast(node: SyntaxNode) -> Option<Self>
            where
                Self: Sized,
            {
                match node.kind() {
                    $($kind => Some(Self(node)),)+
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

syntax_tree_node!(
    /// An inline formula or a display formula
    FormulaItem,
    ItemFormula
);

impl FormulaItem {
    /// Checks whether it is a display formula
    pub fn is_display(&self) -> bool {
        self.syntax().first_token().map_or(false, |node| {
            node.kind() == TokenDollar && node.text() == "$$"
        })
    }

    /// Checks whether it is an inline formula
    pub fn is_inline(&self) -> bool {
        self.syntax().first_token().map_or(false, |node| {
            node.kind() == TokenDollar && node.text() == "$"
        })
    }
}

syntax_tree_node!(
    /// Command item in latex document
    ///
    /// In short it is in shape of
    /// ```coffeescript
    /// ItemCmd(
    ///   ClauseArgument(rev-arg1)?
    ///   ClauseCommandName(name),
    ///   ClauseArgument(arg1), ...
    /// )
    /// ```
    ///
    /// Exmaple:
    /// ```latex
    /// \documentclass{article}
    /// ```
    ///
    /// Which will be parsed as:
    /// ```coffeescript
    /// ItemCmd(
    ///   ClauseCommandName(
    ///    TokenWord("documentclass")
    ///  ),
    ///   ClauseArgument(
    ///     ItemCurly(
    ///      TokenLBrace,
    ///     ItemText(
    ///      TokenWord("article")
    ///    ),
    ///     TokenRBrace
    ///   )
    /// )
    /// ```
    CmdItem,
    ItemCmd
);

impl CmdItem {
    /// Get the token corresponding to command name
    pub fn name_tok(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == ClauseCommandName)
    }

    /// Get the command arguments
    pub fn arguments(&self) -> impl Iterator<Item = SyntaxNode> {
        self.syntax()
            .children()
            .filter(|node| node.kind() == ClauseArgument)
    }
}

syntax_tree_node!(
    /// Environment item in latex document
    /// ```coffeescript
    /// ItemBegin(
    ///   ClauseCommandName(name),
    ///   ClauseArgument(arg1), ...
    /// )
    /// ...
    /// ItemEnd(
    ///   ClauseCommandName(name),
    /// )
    EnvItem,
    ItemEnv
);

impl EnvItem {
    /// Get the begin clause of the environment
    pub fn begin(&self) -> Option<BeginItem> {
        self.syntax().children().find_map(BeginItem::cast)
    }

    /// Get the end clause of the environment
    pub fn end(&self) -> Option<EndItem> {
        self.syntax().children().find_map(EndItem::cast)
    }

    /// Get the name of the environment
    pub fn name_tok(&self) -> Option<SyntaxToken> {
        self.begin().and_then(|begin| begin.name())
    }

    /// Get the arguments of the environment
    pub fn arguments(&self) -> impl Iterator<Item = SyntaxNode> {
        self.begin().into_iter().flat_map(|begin| begin.arguments())
    }
}

syntax_tree_node!(
    /// A paired `\left` and `\right` command with nodes in between them.
    LRItem,
    ItemLR
);

impl LRItem {
    /// Get the left clause
    pub fn left(&self) -> Option<LRClause> {
        self.syntax().first_child().and_then(LRClause::cast)
    }
    /// Get the right clause
    pub fn right(&self) -> Option<LRClause> {
        self.syntax().last_child().and_then(LRClause::cast)
    }

    /// Get the left symbol wrapped in the clause
    pub fn left_sym(&self) -> Option<SyntaxToken> {
        self.left().and_then(|clause| clause.sym())
    }

    /// Get the right symbol wrapped in the clause
    pub fn right_sym(&self) -> Option<SyntaxToken> {
        self.right().and_then(|clause| clause.sym())
    }
}

syntax_tree_node!(
    /// A `\left` or `\right` command
    LRClause,
    ClauseLR
);

impl LRClause {
    /// Get the command kind
    pub fn is_left(&self) -> bool {
        self.syntax()
            .first_token()
            .map(|node| node.kind() == ClauseCommandName && node.text() == "\\left")
            .unwrap_or(false)
    }

    /// Get the symbol wrapped in the clause
    pub fn sym(&self) -> Option<SyntaxToken> {
        self.syntax()
            .last_token()
            .filter(|node| !matches!(node.kind(), ClauseCommandName))
    }
}

syntax_tree_node!(
    /// A `\begin{name}` command with arguments
    BeginItem,
    ItemBegin
);

impl BeginItem {
    /// Get the name in the begin clause
    pub fn name(&self) -> Option<SyntaxToken> {
        self.syntax()
            .first_token()
            .filter(|node| node.kind() == TokenCommandSym)
    }

    /// Get the arguments of the environment
    pub fn arguments(&self) -> impl Iterator<Item = SyntaxNode> {
        self.syntax()
            .children()
            .filter(|node| node.kind() == ClauseArgument)
    }
}

syntax_tree_node!(
    /// A `\end{name}` command
    EndItem,
    ItemEnd
);

impl EndItem {
    /// Get the name in the end clause
    pub fn name(&self) -> Option<SyntaxToken> {
        self.syntax()
            .first_token()
            .filter(|node| node.kind() == TokenCommandSym)
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::SyntaxKind;

    #[test]
    fn test_syntax_repr() {
        use SyntaxKind::*;
        assert_eq!(SyntaxKind::try_from(TokenError as u16), Ok(TokenError));
        assert_eq!(SyntaxKind::try_from(ScopeRoot as u16), Ok(ScopeRoot));
        assert_eq!(SyntaxKind::try_from((ScopeRoot as u16) + 1), Err(()));
    }
}

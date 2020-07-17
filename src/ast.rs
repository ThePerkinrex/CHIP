pub type NameID = String;

macro_rules! ast {
	{$($name:ident($($id:ident : $content:ty),*)),*} => {


		#[allow(dead_code)]
		#[derive(Debug, Clone)]
		pub enum AST {
			$($name($($content),*)),*
		}

		impl AST {
			#[allow(unused_variables)]
			pub fn as_kind(&self) -> StatementKind {
				match self {
					$(Self::$name($($id),*)=>StatementKind::$name),*
				}
			}
		}

		#[allow(dead_code)]
		#[derive(Debug, Clone, PartialEq)]
		pub enum StatementKind {
			$($name),*
		}
	};
}

ast! {
	USE(path: NameID, alias: NameID), // CHIP names available for creation could have other ids
	IN(n: NameID),
	OUT(n: NameID),

	RAIL(n: NameID),

	CHIP(alias: NameID, name: NameID),

	CONNECT(a: NameID, b: NameID),

	CUSTOM(n: NameID)
}

pub trait Element {
    fn get_name(&self) -> &String;
    fn get_attributes(&self) -> &Vec<String>;
}

pub trait Text {
    fn get_value(&self) -> &String;
}

pub enum Node<TElem: Element, TText: Text> {
    Element(TElem),
    Text(TText),
}

pub trait Document {
    type TElem : Element;
    type TText: Text;

    fn get_root(&self) -> &Node<Self::TElem, Self::TText>;
    fn get_children_of(&self, element: &Self::TElem) -> Vec<&Node<Self::TElem, Self::TText>>;
    fn get_parent_of(&self, node: &Node<Self::TElem, Self::TText>) -> Option<&Node<Self::TElem, Self::TText>>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use bip329::{Label as Bip329Label, Labels};

use crate::{
    annotation::{Annotation, RecordType},
    label::Label,
};

impl From<Bip329Label> for Annotation {
    fn from(bip329: Bip329Label) -> Self {
        let (record_type, ref_, origin, spendable) = match &bip329 {
            Bip329Label::Transaction(r) => {
                (RecordType::Tx, r.ref_.to_string(), r.origin.clone(), None)
            }
            Bip329Label::Address(r) => {
                (RecordType::Addr, format!("{}", r.ref_.clone().assume_checked()), None, None)
            }
            Bip329Label::PublicKey(r) => {
                (RecordType::Pubkey, r.ref_.clone(), None, None)
            }
            Bip329Label::Input(r) => {
                (RecordType::Input, r.ref_.to_string(), None, None)
            }
            Bip329Label::Output(r) => {
                (RecordType::Output, r.ref_.to_string(), None, Some(r.spendable))
            }
            Bip329Label::ExtendedPublicKey(r) => {
                (RecordType::Xpub, r.ref_.clone(), None, None)
            }
        };

        let label_str = bip329.label().unwrap_or("");
        let parsed = Label::parse(label_str);

        Annotation {
            record_type,
            ref_,
            description: parsed.description,
            origin,
            spendable,
            tags: parsed.tags,
        }
    }
}

impl TryFrom<Annotation> for Bip329Label {
    type Error = anyhow::Error;

    fn try_from(annotation: Annotation) -> Result<Self, Self::Error> {
        let label_str = Label {
            description: annotation.description,
            tags: annotation.tags,
        }
        .to_bip329_string();

        Ok(match annotation.record_type {
            RecordType::Tx => {
                let ref_ = annotation.ref_.parse::<bitcoin::Txid>()
                    .map_err(|e| anyhow::anyhow!("invalid txid '{}': {e:?}", annotation.ref_))?;
                Bip329Label::Transaction(bip329::TransactionRecord {
                    ref_,
                    label: label_str,
                    origin: annotation.origin,
                })
            }
            RecordType::Addr => {
                let ref_ = annotation.ref_.parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
                    .map_err(|e| anyhow::anyhow!("invalid address '{}': {e:?}", annotation.ref_))?;
                Bip329Label::Address(bip329::AddressRecord {
                    ref_,
                    label: label_str,
                })
            }
            RecordType::Pubkey => {
                Bip329Label::PublicKey(bip329::PublicKeyRecord {
                    ref_: annotation.ref_,
                    label: label_str,
                })
            }
            RecordType::Input => {
                let ref_ = annotation.ref_.parse::<bitcoin::OutPoint>()
                    .map_err(|e| anyhow::anyhow!("invalid input outpoint '{}': {e:?}", annotation.ref_))?;
                Bip329Label::Input(bip329::InputRecord {
                    ref_,
                    label: label_str,
                })
            }
            RecordType::Output => {
                let ref_ = annotation.ref_.parse::<bitcoin::OutPoint>()
                    .map_err(|e| anyhow::anyhow!("invalid output outpoint '{}': {e:?}", annotation.ref_))?;
                Bip329Label::Output(bip329::OutputRecord {
                    ref_,
                    label: label_str,
                    spendable: annotation.spendable.unwrap_or(true),
                })
            }
            RecordType::Xpub => {
                Bip329Label::ExtendedPublicKey(bip329::ExtendedPublicKeyRecord {
                    ref_: annotation.ref_,
                    label: label_str,
                })
            }
        })
    }
}

pub fn annotations_from_bip329(labels: Labels) -> Vec<Annotation> {
    labels.into_vec().into_iter().map(Annotation::from).collect()
}

pub fn annotations_to_bip329(annotations: Vec<Annotation>) -> anyhow::Result<Labels> {
    let labels = annotations
        .into_iter()
        .map(Bip329Label::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Labels::new(labels))
}

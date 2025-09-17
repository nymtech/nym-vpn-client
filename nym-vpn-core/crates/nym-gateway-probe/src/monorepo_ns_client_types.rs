use nym_credentials::ecash::bandwidth::serialiser::{VersionSerialised, VersionedSerialise};
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
    Error, IssuedTicketBook,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Serialize, Deserialize)]
pub struct AttachedTicket {
    pub ticketbook: VersionSerialised<IssuedTicketBook>,
    pub usable_index: u32,
}

#[derive(Deserialize, Serialize)]
pub struct AttachedTicketMaterials {
    pub coin_indices_signatures: Vec<VersionSerialised<AggregatedCoinIndicesSignatures>>,

    pub expiration_date_signatures: Vec<VersionSerialised<AggregatedExpirationDateSignatures>>,

    pub master_verification_keys: Vec<VersionSerialised<EpochVerificationKey>>,

    // we need one ticket per type
    pub attached_tickets: Vec<AttachedTicket>,
}

impl AttachedTicketMaterials {
    pub fn to_serialised_string(&self) -> String {
        // TODO: we're losing revision here, but given we control both ends of the pipeline,
        // that's fine. we can just pass it as a separate argument
        let serialised = self.pack();
        bs58::encode(serialised.data).into_string()
    }

    pub fn from_serialised_string(raw: &str, revision: u8) -> Result<Self, Error> {
        let bytes = bs58::decode(raw)
            .into_vec()
            .inspect_err(|err| error!("malformed bytes encoding: {err}"))
            .unwrap_or_default();
        Self::try_unpack(&bytes, revision)
    }
}

impl VersionedSerialise for AttachedTicketMaterials {
    const CURRENT_SERIALISATION_REVISION: u8 = 1;

    fn try_unpack(b: &[u8], revision: impl Into<Option<u8>>) -> Result<Self, Error>
    where
        Self: DeserializeOwned,
    {
        let revision = revision
            .into()
            .unwrap_or(<Self as VersionedSerialise>::CURRENT_SERIALISATION_REVISION);

        match revision {
            1 => Self::try_unpack_current(b),
            _ => Err(Error::UnknownSerializationRevision { revision }),
        }
    }
}

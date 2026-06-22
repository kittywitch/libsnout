use std::net::ToSocketAddrs;

use crate::{
    calibration::{EyeShape, FaceShape, Weights}, config::Config, output::{BabbleEmitter, EtvrEmitter, NativeEmitter, OscTransport, TransportError},
};

pub struct Output {
    pub transport: Option<OscTransport>,
    pub babble: BabbleEmitter,
    pub etvr: EtvrEmitter,
    pub native: NativeEmitter,
    pub native_transport: Option<OscTransport>,
}

impl Output {
    pub fn new() -> Self {
        Self {
            transport: None,
            babble: BabbleEmitter::new(),
            etvr: EtvrEmitter::new(),
            native: NativeEmitter::new(),
            native_transport: None,
        }
    }

    pub fn with_config(config: &Config) -> Result<Self, TransportError> {
        let mut output = Self::new();

        output.set_destination(&config.output.osc.destination)?;

        if let Some(native) = &config.output.native {
            output.set_native_destination(&native.destination)?;
        }

        Ok(output)
    }

    pub fn set_destination(
        &mut self,
        destination: impl ToSocketAddrs,
    ) -> Result<(), TransportError> {
        self.transport = Some(OscTransport::udp(destination)?);
        Ok(())
    }

    pub fn set_native_destination(
        &mut self,
        destination: impl ToSocketAddrs,
    ) -> Result<(), TransportError> {
        self.native_transport = Some(OscTransport::udp(destination)?);
        Ok(())
    }

    pub fn send_face(&mut self, weights: Weights<'_, FaceShape>) {
        let Some(transport) = &mut self.transport else {
            return;
        };

        self.babble.process_face(weights, transport);
    }

    pub fn send_eyes(&mut self, weights: Weights<'_, EyeShape>) {
        let Some(transport) = &mut self.transport else {
            return;
        };

        self.babble.process_eyes(weights, transport);
        self.etvr.process_eyes(weights, transport);

        if let Some(native_transport) = &mut self.native_transport {
            self.native.process_eyes(weights, native_transport);
        }
    }

    pub fn flush(&mut self) -> Result<(), TransportError> {
        let Some(transport) = &mut self.transport else {
            return Ok(());
        };

        transport.flush()
    }
}

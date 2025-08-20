
/**
* @typedef {{
  t: "FinishInit",
  c: import("mediasoup-client").types.RtpCapabilities
}} FinishInit
*/

/**
* @typedef {{
  t: "ConnectProducerTransport",
  c: import('mediasoup-client').types.DtlsParameters
}} ConnectProducerTransport
*/

/**
* @typedef {{
  t: "Produce",
  c: [import("mediasoup-client").types.MediaKind, import("mediasoup-client").types.RtpParameters]
}} Produce
*/

/**
* @typedef {{
  t: "ConnectConsumerTransport",
  c: import('mediasoup-client').types.DtlsParameters
}} ConnectConsumerTransport
*/

/**
* @typedef {{
  t: "Consume",
  c: string
}} Consume
*/

/**
* @typedef {{
  t: "ConsumerResume",
  c: string
}} ConsumerResume
*/

/**
* @typedef {FinishInit|ConnectProducerTransport|Produce|ConnectConsumerTransport|Consume|ConsumerResume} MediaSoup
*/

export { }

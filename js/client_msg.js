
/**
* @typedef {{
  t: "FinishInit",
  d: import("mediasoup-client").types.RtpCapabilities
}} FinishInit
*/

/**
* @typedef {{
  t: "ConnectProducerTransport",
  d: import('mediasoup-client').types.DtlsParameters
}} ConnectProducerTransport
*/

/**
* @typedef {{
  t: "Produce",
  d: [import("mediasoup-client").types.MediaKind, import("mediasoup-client").types.RtpParameters]
}} Produce
*/

/**
* @typedef {{
  t: "ConnectConsumerTransport",
  d: import('mediasoup-client').types.DtlsParameters
}} ConnectConsumerTransport
*/

/**
* @typedef {{
  t: "Consume",
  d: string
}} Consume
*/

/**
* @typedef {{
  t: "ConsumerResume",
  d: string
}} ConsumerResume
*/

/**
* @typedef {FinishInit|ConnectProducerTransport|Produce|ConnectConsumerTransport|Consume|ConsumerResume} MediaSoup
*/

export { }

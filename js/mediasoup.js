
/** @type {import("mediasoup-client").Device} */
const device = new window.mediasoupClient.Device()

/**
* @type {{
  send: (any) => void,
  recv: () => Promise<any>
}}
*/
var dioxus;

/**
* @type {import("mediasoup-client").types.Transport}
*/
let producerTransport

/**
* @type {import("mediasoup-client").types.Transport}
*/
let consumerTransport

/**
* @typedef {{
  t: "SetRoom",
  d: {
    room_id: string,
    consumer_transport_options: import("mediasoup-client").types.TransportOptions,
    producer_transport_options: import("mediasoup-client").types.TransportOptions,
    router_rtp_capabilities: import("mediasoup-client").types.RtpCapabilities,
    producers: [string, string][]
  }
}} SetRoom
*/

/**
* @typedef {{
  t: "ConnectProducerTransport",
  d: {}
}} ConnectProducerTransport
*/

/**
* @typedef {{
  t: "Produce",
  d: string
}} Produce
*/

/**
* @typedef {SetRoom | ConnectProducerTransport | Produce} WebsocketServerResData
*/

while (true) {
  /**
  * @type {WebsocketServerResData}
  */
  const msg = await dioxus.recv()

  switch (msg.t) {
    case "SetRoom":
      const set_room_data = msg.d;
    
    break;
  }
}



// @ts-ignore
/** @type {import("mediasoup-client").Device} */
// @ts-ignore
const device = new window.mediasoupClient.Device()


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
* @typedef {{
  t: "ConnectConsumerTransport",
  d: {}
}} ConnectConsumerTransport
*/

/**
* @typedef {{
  t: "ProducerAdded",
  d: {
    participant_id: string,
    producer_id: string,
  }
}} ProducerAdded
*/

/**
* @typedef {{
  t: "Consume",
  d: {
    id: String,
    producer_id: String,
    kind: import("mediasoup-client").types.MediaKind,
    rtp_parameters: import("mediasoup-client").types.RtpParameters,
  }
}} Consume
*/

/** @typedef {{
  t: "ProducerRemove",
  d: {
    participant_id: string,
    producer_id: string,
  }
}} ProducerRemove

*/
/**
* @typedef {SetRoom | ConnectProducerTransport | Produce | ConnectConsumerTransport | ProducerAdded | Consume | ProducerRemove} WebsocketServerResData
*/

/**
* @type {{
  send: (msg: import('client_msg').MediaSoup) => void,
  recv: () => Promise<any>
}}
*/
var dioxus;

/**
* @type {Map<WebsocketServerResData['t'], (cv: any) => void>}
*/
const listeners = new Map()

async function mediasoup() {
  while (true) {
    /**
    * @type {WebsocketServerResData}
    */
    const msg = await dioxus.recv()

    const cb = listeners.get(msg.t)
    if (cb) {
      listeners.delete(msg.t)
      cb(msg.d)

      // skip switch case
      continue
    }

    switch (msg.t) {
      case "SetRoom":
        const set_room_data = msg.d;

        await device.load({
          routerRtpCapabilities: set_room_data.router_rtp_capabilities
        });

        /**
        * @type {import('./client_msg').FinishInit}
        */
        const finishInit = {
          t: "FinishInit",
          d: device.rtpCapabilities
        }

        dioxus.send(finishInit)

        producerTransport = device.createSendTransport(
          set_room_data.producer_transport_options
        )

        producerTransport
          .on('connect', ({ dtlsParameters }, success) => {
            /**
            * @type {import('./client_msg').ConnectProducerTransport}
            */
            const connectProducerTransport = {
              t: "ConnectProducerTransport",
              d: dtlsParameters
            }

            dioxus.send(connectProducerTransport)

            listeners.set('ConnectProducerTransport', () => {
              success()
            })
          })
          .on('produce', ({ kind, rtpParameters }, success) => {
            /**
            * @type {import('./client_msg').Produce}
            */
            const produce = {
              t: "Produce",
              d: [kind, rtpParameters]
            }

            dioxus.send(produce)

            listeners.set("Produce", (id) => {
              success({ id })
            })
          })

        const mediaStream = await navigator.mediaDevices.getUserMedia({
          audio: true,
          video: {
            width: {
              ideal: 1280
            },
            height: {
              ideal: 720
            },
            frameRate: {
              ideal: 60
            }
          }
        })

        // sendPreview.srcObject = mediaStream;

        for (const track of mediaStream.getTracks()) {
          const producer = await producerTransport.produce({ track })
        }

        consumerTransport = device.createRecvTransport(set_room_data.consumer_transport_options)

        consumerTransport.on('connect', ({ dtlsParameters }, success) => {
          /**
          * @type {import("./client_msg").ConnectConsumerTransport}
          */
          const connectConsumerTransport = {
            t: "ConnectConsumerTransport",
            d: dtlsParameters
          }

          dioxus.send(connectConsumerTransport)

          listeners.set("ConnectConsumerTransport", () => {
            success()
          })
        })

        break;

      case "ProducerAdded":
        const producer_added_data = msg.d;

        await new Promise((resolve) => {
          /**
          * @type {import("./client_msg").Consume}
          */
          const consume = {
            t: "Consume",
            d: producer_added_data.producer_id
          }

          dioxus.send(consume)

          listeners.set('Consume', async (d) => {
            /**
            * @type {Consume['d']}
            */
            let consumer_data = d

            const consumer = await consumerTransport.consume({
              id: consumer_data.id,
              producerId: consumer_data.producer_id,
              kind: consumer_data.kind,
              rtpParameters: consumer_data.rtp_parameters,
            })

            /**
            * @type {import("client_msg").ConsumerResume}
            */
            const consumer_resume = {
              t: "ConsumerResume",
              d: consumer.id
            }

            dioxus.send(consumer_resume)

            //      participants
            // .addTrack(message.participantId, message.producerId, consumer.track);
            resolve(undefined);
          })
        })

        break;

      case "ProducerRemove":
        //    participants
        // .deleteTrack(message.participantId, message.producerId);

        break;

      default:
        console.error("Received unexpected message", msg)
    }
  }
}

mediasoup()


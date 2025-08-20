
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

/**
* @typedef {{
  t: "ConsumerResume",
  d: {}
}} ConsumerResume
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
* @typedef {SetRoom | ConnectProducerTransport | Produce | ConnectConsumerTransport | Consume | ConsumerResume} WebsocketServerResData
*/

/**
* @template V, E
* @typedef {{ Ok: V } | { Err: E }} Result<V, E>
*/

/**
* @typedef {{
  t: "RequestResponse",
  c: {
    id: string,
    data: Result<WebsocketServerResData, string>
  }
}} RequestResponse
*/

/**
* @typedef {ProducerAdded | ProducerRemove | RequestResponse} WebsocketServerMessage 
*/

/*
* @type {{
  send: (msg: import('client_msg').MediaSoup) => void,
  recv: () => Promise<WebsocketServerMessage>
}}
*/


/**
* @param {import('client_msg').MediaSoup} msg 
*/
function send(msg) {
  // @ts-ignore
  dioxus.send(msg)
}

/**
* @returns {Promise<WebsocketServerMessage>}
*/
async function recv() {
  // @ts-ignore
  return dioxus.recv()
}

/**
* @type {Map<WebsocketServerResData['t'], (cv: any) => void>}
*/
const listeners = new Map()


/**
* @overload
* @param {import('client_msg').FinishInit} msg 
* @returns {Promise<Result<import("client_msg").FinishInit, string>>}
*/

/**
* @overload
* @param {import('client_msg').ConnectProducerTransport} msg 
* @returns {Promise<Result<import("client_msg").ConnectProducerTransport, string>>}
*/

/**
* @overload
* @param {import('client_msg').Produce} msg 
* @returns {Promise<Result<Produce, string>>}
*/

/**
* @overload
* @param {import('client_msg').ConnectConsumerTransport} msg 
* @returns {Promise<Result<import("client_msg").ConnectConsumerTransport, string>>}
*/

/**
* @overload
* @param {import('client_msg').Consume} msg 
* @returns {Promise<Result<Consume, string>>}
*/

/**
* @overload
* @param {import('client_msg').ConsumerResume} msg 
* @returns {Promise<Result<ConsumerResume, string>>}
*/

/**
* @param {import('client_msg').MediaSoup} msg 
* @returns {Promise<Result<WebsocketServerResData, string>>}
*/
async function ws_request(msg) {
  return new Promise((resolve) => {
    send(msg)

    // @ts-ignore
    listeners.set(msg.t, resolve)
  })
}

class Participant {
  /**
  * @param {string} id 
  */
  constructor(id) {
    const container = document.querySelector("#container")

    if (!container) {
      return;
    }


    /**
    * @type {MediaStream}
    */
    this.mediaStream = new MediaStream()
    /**
    * @type {HTMLElement}
    */
    this.figure = document.createElement('figure')
    /**
    * @type {HTMLVideoElement}
    */
    this.preview = document.createElement('video')

    this.preview.muted = true
    this.preview.controls = true

    this.preview.onloadedmetadata = () => {
      this.preview.play()
    }

    const figcaption = document.createElement('figcaption')

    figcaption.innerText = `Participant ${id}`

    this.figure.append(this.preview, figcaption)

    container.append(this.figure)
  }

  /**
  * @param {MediaStreamTrack} track 
  */
  addTrack(track) {
    this.mediaStream.addTrack(track)
    this.preview.srcObject = this.mediaStream
  }

  /**
  * @param {MediaStreamTrack} track 
  */
  removeTrack(track) {
    this.mediaStream.removeTrack(track)
    this.preview.srcObject = this.mediaStream
  }

  hasTracks() {
    return this.mediaStream.getTracks().length > 0
  }

  destroy() {
    this.preview.srcObject = null
    this.figure.remove()
  }
}

class Participants {
  constructor() {
    /**
    * @type {Map<string, Participant>}
    */
    this.participants = new Map()
    /**
    * @type {Map<string, MediaStreamTrack>}
    */
    this.producerIdToTrack = new Map()
  }

  /**
  * @param {string} participantId 
  * @param {string} producerId
  * @param {MediaStreamTrack} track 
  */
  addTrack(
    participantId,
    producerId,
    track
  ) {
    this.producerIdToTrack.set(producerId, track)
    this.getOrCreateParticipant(participantId).addTrack(track)
  }

  /**
  * @param {string} participantId 
  * @param {string} producerId
  */
  removeTrack(
    participantId,
    producerId,

  ) {
    const track = this.producerIdToTrack.get(producerId)

    if (!track) {
      return
    }

    const participant = this.getOrCreateParticipant(participantId)

    participant.removeTrack(track)

    if (!participant.hasTracks()) {
      this.participants.delete(participantId)
      participant.destroy
    }
  }

  /**
  * @param {string} id 
  * @returns {Participant}
  */
  getOrCreateParticipant(id) {
    const participant = this.participants.get(id)

    if (participant) {
      return participant
    }

    const newParticipant = new Participant(id)
    this.participants.set(id, newParticipant)

    return newParticipant
  }
}

const participants = new Participants()

async function mediasoup() {
  while (true) {
    const msg = await recv()

    switch (msg.t) {
      case "RequestResponse":
        const data = msg.c.data;

        if ("Ok" in data) {
          if (data.Ok.t === "SetRoom") {
            // TODO: clear?

            console.log(msg)

            const set_room_data = data.Ok.d;

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

            await ws_request(finishInit)

            producerTransport = device.createSendTransport(
              set_room_data.producer_transport_options
            )

            producerTransport
              .on('connect', async ({ dtlsParameters }, success, error) => {
                /**
                * @type {import('./client_msg').ConnectProducerTransport}
                */
                const connectProducerTransport = {
                  t: "ConnectProducerTransport",
                  d: dtlsParameters
                }

                const r = await ws_request(connectProducerTransport)
                if ("Ok" in r) {
                  success()
                } else {
                  error(new Error(r.Err))
                }
              })
              .on('produce', async ({ kind, rtpParameters }, success, error) => {
                /**
                * @type {import('./client_msg').Produce}
                */
                const produce = {
                  t: "Produce",
                  d: [kind, rtpParameters]
                }

                const r = await ws_request(produce)
                if ("Ok" in r) {
                  success({ id: r.Ok.d })
                } else {
                  error(new Error(r.Err))
                }
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
              await producerTransport.produce({ track })
            }

            consumerTransport = device.createRecvTransport(set_room_data.consumer_transport_options)

            consumerTransport.on('connect', async ({ dtlsParameters }, success, error) => {
              /**
              * @type {import("./client_msg").ConnectConsumerTransport}
              */
              const connectConsumerTransport = {
                t: "ConnectConsumerTransport",
                d: dtlsParameters
              }

              const r = await ws_request(connectConsumerTransport)

              if ("Ok" in r) {
                success()
              } else {
                error(new Error(r.Err))
              }
            })
          }

          const cb = listeners.get(data.Ok.t)

          if (cb) {
            listeners.delete(data.Ok.t)
            cb(data.Ok.d)
          }
        } else {
          console.error(data)
        }

        break

      case "ProducerAdded":
        const producer_added_data = msg.d;

        await new Promise(async (resolve, reject) => {
          /**
          * @type {import("./client_msg").Consume}
          */
          const consume = {
            t: "Consume",
            d: producer_added_data.producer_id
          }

          const r = await ws_request(consume)
          if ("Err" in r) {
            return reject(r.Err)
          }

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

            const r = await ws_request(consumer_resume)
            if ("Ok" in r) {
              participants
                .addTrack(consumer_data.id, consumer_data.producer_id, consumer.track);
              resolve(undefined);
            } else {
              reject(r.Err)
            }
          })
        })

        break;

      case "ProducerRemove":
        const producer_remove_data = msg.d;
        participants
          .removeTrack(producer_remove_data.participant_id, producer_remove_data.producer_id);

        break;

      default:
        console.error("Received unexpected message", msg)
    }
  }
}

// @ts-ignore
await mediasoup()
  .catch(console.error)


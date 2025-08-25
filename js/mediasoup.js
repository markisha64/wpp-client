
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
  c: {
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
  c: {}
}} ConnectProducerTransport
*/

/**
* @typedef {{
  t: "Produce",
  c: string
}} Produce
*/

/**
* @typedef {{
  t: "ConnectConsumerTransport",
  c: {}
}} ConnectConsumerTransport
*/

/**
* @typedef {{
  t: "ProducerAdded",
  c: {
    participant_id: string,
    producer_id: string,
  }
}} ProducerAdded
*/

/**
* @typedef {{
  t: "Consume",
  c: {
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
  c: {}
}} ConsumerResume
*/

/** @typedef {{
  t: "ProducerRemove",
  c: {
    participant_id: string,
    producer_id: string,
  }
}} ProducerRemove
*/

/**
* @typedef {{
  t: "FinishInit",
  c: string
}} FinishInit
*/

/**
* @typedef {{
  t: "LeaveRoom"
}} LeaveRoom
*/

/**
* @typedef {SetRoom | ConnectProducerTransport | Produce | ConnectConsumerTransport | Consume | ConsumerResume | FinishInit | LeaveRoom} MediaSoupResponse
*/

/**
* @template T
* @typedef {{
  t: "MS",
  c: T
}} MS<T>
*/

/**
* @typedef {MS<MediaSoupResponse>} WebsocketServerResData
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
* @param {import("client_msg").MediaSoup} msg 
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
* @type {Map<MediaSoupResponse['t'], (cv: any) => void>}
*/
const listeners = new Map()


/**
* @overload
* @param {import('client_msg').FinishInit} msg 
* @returns {Promise<Result<MS<FinishInit>, string>>}
*/

/**
* @overload
* @param {import('client_msg').ConnectProducerTransport} msg 
* @returns {Promise<Result<MS<ConnectProducerTransport>, string>>}
*/

/**
* @overload
* @param {import('client_msg').Produce} msg 
* @returns {Promise<Result<MS<Produce>, string>>}
*/

/**
* @overload
* @param {import('client_msg').ConnectConsumerTransport} msg 
* @returns {Promise<Result<MS<ConnectConsumerTransport>, string>>}
*/

/**
* @overload
* @param {import('client_msg').Consume} msg 
* @returns {Promise<Result<MS<Consume>, string>>}
*/

/**
* @overload
* @param {import('client_msg').ConsumerResume} msg 
* @returns {Promise<Result<MS<ConsumerResume>, string>>}
*/

/**
* @param {import('client_msg').MediaSoup} msg 
* @returns {Promise<Result<MS<MediaSoupResponse>, string>>}
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
    const container = document.querySelector("#media-sources")

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
    this.figure = document.createElement('figure', {})
    this.figure.classList.add("flex-[0_1_320px]", "min-w-[220px]", "max-w-[360px]", "max-w-full")

    const wrapper = document.createElement("div")
    wrapper.classList.add("relative", "rounded-xl", "overflow-hidden", "bg-black", "ring-1", "ring-white/10", "shadow-lg")

    /**
    * @type {HTMLVideoElement}
    */
    this.preview = document.createElement('video')
    this.preview.classList.add("block", "w-full", "aspect-video", "object-cover")
    this.preview.muted = true

    this.preview.onloadedmetadata = () => {
      this.preview.play()
    }

    const figcaption = document.createElement('figcaption')

    figcaption.classList.add("mt-2", "text-center", "text-sm", "text-white/70")
    figcaption.innerText = `Participant ${id}`

    wrapper.append(this.preview, figcaption)

    this.figure.append(wrapper)

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
      participant.destroy()
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

/**
* @param {ProducerAdded} msg 
*/
async function producerAdded(msg) {
  const producer_added_data = msg.c;

  await new Promise(async (resolve, reject) => {
    /**
    * @type {import("./client_msg").Consume}
    */
    const consume = {
      t: "Consume",
      c: producer_added_data.producer_id
    }

    const r = await ws_request(consume)
    if ("Err" in r) {
      return reject(r.Err)
    }

    /**
    * @type {Consume['c']}
    */
    let consumer_data = r.Ok.c.c

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
      c: consumer.id
    }

    const r1 = await ws_request(consumer_resume)
    if ("Ok" in r1) {
      participants
        .addTrack(producer_added_data.participant_id, producer_added_data.producer_id, consumer.track);
      resolve(undefined);
    } else {
      reject(r1.Err)
    }
  })
}

/**
* @param {WebsocketServerMessage} msg 
*/
async function mediasoupHandler(msg) {
  switch (msg.t) {
    case "RequestResponse":
      const data = msg.c.data;

      if ("Ok" in data) {
        if (data.Ok.c.t === "LeaveRoom") {
          for (const participant of participants.participants.values()) {
            participant.destroy()
          }

          participants.participants.clear()
          listeners.clear()

          producerTransport.close()
          consumerTransport.close()
        }

        if (data.Ok.c.t === "SetRoom") {
          // TODO: clear?

          const set_room_data = data.Ok.c.c;

          await device.load({
            routerRtpCapabilities: set_room_data.router_rtp_capabilities
          });

          /**
          * @type {import('./client_msg').FinishInit}
          */
          const finishInit = {
            t: "FinishInit",
            c: device.rtpCapabilities
          }

          const ice_servers_string = await ws_request(finishInit)
          if ("Err" in ice_servers_string) {
            return;
          }

          producerTransport = device.createSendTransport({
            ...set_room_data.producer_transport_options,
            ...JSON.parse(ice_servers_string.Ok.c.c)
          })

          producerTransport
            .on('connect', async ({ dtlsParameters }, success, error) => {
              /**
              * @type {import('./client_msg').ConnectProducerTransport}
              */
              const connectProducerTransport = {
                t: "ConnectProducerTransport",
                c: dtlsParameters
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
                c: [kind, rtpParameters]
              }

              const r = await ws_request(produce)
              if ("Ok" in r) {
                success({ id: r.Ok.c.c })
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

          /**
          * @type {HTMLVideoElement | null}
          */
          const sendPreview = document.querySelector("#preview-send")
          if (sendPreview) {
            sendPreview.srcObject = mediaStream;
            sendPreview.onloadedmetadata = () => {
              // dunno why but didn't work how i wanted until this
              sendPreview.muted = true
              sendPreview.play()
            }
          }

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
              c: dtlsParameters
            }

            const r = await ws_request(connectConsumerTransport)

            if ("Ok" in r) {
              success()
            } else {
              error(new Error(r.Err))
            }
          })

          for (const [participant_id, producer_id] of set_room_data.producers) {
            await producerAdded({
              t: "ProducerAdded",
              c: {
                participant_id,
                producer_id
              }
            })
          }
        }
      } else {
        console.error(data)
      }

      break

    case "ProducerAdded":
      await producerAdded(msg)

      break;

    case "ProducerRemove":
      const producer_remove_data = msg.c;
      participants
        .removeTrack(producer_remove_data.participant_id, producer_remove_data.producer_id);

      break;

    default:
      console.error("Received unexpected message", msg)
  }
}

async function mediasoup() {
  while (true) {
    const msg = await recv()

    console.log(msg)

    if (msg.t === "RequestResponse") {
      const data = msg.c.data;

      if ("Ok" in data) {
        if (data.Ok.c.t !== "SetRoom") {
          const cb = listeners.get(data.Ok.c.t)

          if (cb) {
            listeners.delete(data.Ok.c.t)
            cb(data)
          }

          continue
        }
      } else {
        console.error(msg)
        continue
      }
    }

    mediasoupHandler(msg)
      .catch(console.error)
  }
}

setInterval(() => { }, 2000)

// @ts-ignore
await mediasoup()
  .catch(console.error)


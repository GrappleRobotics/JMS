import { AudienceDisplaySound } from "../ws-schema";

export const playSound = async (sound: AudienceDisplaySound) => {
  console.log("Playing Sound: " + sound);
  const audio = new Audio(`/sounds/${sound}.wav`);
  audio.play().catch((e: DOMException) => {
    if (e.message.includes("interact")) {
      alert("Can't play sound - autoplay policy. Interact with this page first!");
    }
  })
}
import React from "react";
import { Composition } from "remotion";
import { Film } from "./Film";
import { FPS, FRAMES, W, H } from "./lib/consts";

export const RemotionRoot: React.FC = () => {
  return (
    <Composition
      id="NotToNoticeV3"
      component={Film}
      durationInFrames={FRAMES}
      fps={FPS}
      width={W}
      height={H}
    />
  );
};

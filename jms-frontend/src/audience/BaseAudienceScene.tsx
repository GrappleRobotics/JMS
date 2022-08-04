import React from "react";
import { WebsocketComponent } from "support/ws-component";
import { EventDetails } from "ws-schema";

export type AudienceSceneBaseProps<T> = {
  details: EventDetails,
  props?: T
};

export default abstract class BaseAudienceScene<P={}, S={}> extends WebsocketComponent<AudienceSceneBaseProps<P>, S> {
  abstract show: (props: P) => React.ReactNode;
  onShow = () => {};
  onHide = () => {};

  componentDidUpdate = (prevProps: AudienceSceneBaseProps<P>) => {
    if (prevProps.props != null && this.props.props == null)
      this.onShow();
    if (!prevProps.props == null && this.props.props != null)
      this.onHide();
  }
  
  render = () => {
    return this.props.props != null ? this.show(this.props.props!) : <React.Fragment />
  }
}

export class AudienceSceneField extends BaseAudienceScene {
  show = () => {
    return <div className="audience-field" />
  }
}
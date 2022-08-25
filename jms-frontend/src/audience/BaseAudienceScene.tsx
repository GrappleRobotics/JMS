import React from "react";
import { CSSTransition } from "react-transition-group";
import { WebsocketComponent } from "support/ws-component";
import { EventDetails } from "ws-schema";

export type AudienceSceneBaseProps<T> = {
  details: EventDetails,
  props?: T
};

export type AudienceSceneBaseState<P, S> = {
  _props?: P,
  _show?: boolean
} & S;

export default abstract class BaseAudienceScene<P={}, S={}> extends WebsocketComponent<AudienceSceneBaseProps<P>, AudienceSceneBaseState<P, S>> {
  readonly state: AudienceSceneBaseState<P, S> = {} as any;
  
  abstract show: (props: P) => React.ReactNode;
  onShow = () => {};
  onHide = () => {};
  onUpdate = (prevProps: AudienceSceneBaseProps<P>, prevState: S) => {};

  // Latch previous props so the animation can take place
  static getDerivedStateFromProps(nextProps: AudienceSceneBaseProps<any>, prevState: AudienceSceneBaseState<any, any>) {
    if (nextProps.props == null && prevState._props != null) {
      return { _show: false };
    } else {
      return { _show: nextProps.props != null, _props: nextProps.props };
    }
  }

  componentDidUpdate = (prevProps: AudienceSceneBaseProps<P>, prevState: S) => {
    if (prevProps.props != null && this.props.props == null) 
      this.onShow();
    if (!prevProps.props == null && this.props.props != null)
      this.onHide();
    this.onUpdate(prevProps, prevState);
  }
  
  render = () => {
    // const { props, details, ...others } = this.props;

    return <CSSTransition
      key={this.constructor.name}
      in={this.state._show}
      onExited={() => { this.setState({ _props: undefined }) }}
      classNames="audience-scene-anim"
      timeout={500}
    >
      <div className="audience-scene-anim">
        { this.state._props != null ? this.show(this.state._props!) : undefined }
        {/* { props != null ? this.show(props!) : ( this.lastProps != null ? this.show(this.lastProps!) : <React.Fragment /> ) } */}
      </div>
    </CSSTransition>
      {/* {
        props != null ? this.show(props!) : <React.Fragment />
      } */}
  }
}

export class AudienceSceneField extends BaseAudienceScene {
  show = () => {
    return <div className="audience-field" />
  }
}
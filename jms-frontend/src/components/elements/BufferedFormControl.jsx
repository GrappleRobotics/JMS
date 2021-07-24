import React from 'react';
import { FormControl } from 'react-bootstrap';

export default class BufferedFormControl extends React.Component {
  constructor(props) {
    super(props);
    
    this.state = {
      value: this.props.value
    }

    this.timer = null;

    this.controlRef = React.createRef();
  }

  // Make sure outside updates propagate here
  componentDidUpdate(prevProps) {
    if (prevProps.value !== this.props.value) {
      this.setState({ value: this.props.value || "" });    // Have to || "" otherwise the FormControl won't update
    }
  }

  componentDidMount() {
    if (this.props.autofocus) {
      this.controlRef.current.focus();
    }
  }

  valuesDiffer = () => {
    let oldV = String(this.props.value || "");
    let newV = String(this.state.value || "");

    return oldV !== newV;
  }

  changed = (v) => {
    if (this.props.auto) {
      clearTimeout(this.timer);
      this.timer = setTimeout(() => this.props.onUpdate(this.state.value), this.props.autoMillis || 250);
    }
    this.setState({ value: v })
  }

  focusInput = () => {
    this.controlRef.current.focus();
  }

  render() {
    let { className, onUpdate, value, ...props } = this.props;

    return <FormControl
      ref={this.controlRef}
      className={ (className || "") + " " + (this.valuesDiffer() ? "buffer-diff" : "buffer-same") }
      value={this.state.value}
      onChange={(v) => this.changed(v.target.value)}
      onKeyDown={(e) => {
        if (e.key === 'Enter') {
          onUpdate(this.state.value);
        }
      }}
      {...props}
    />
  }
}
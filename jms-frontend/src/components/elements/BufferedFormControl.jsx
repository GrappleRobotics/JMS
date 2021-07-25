import React from 'react';
import { FormControl } from 'react-bootstrap';
import Buffered from './Buffered';

export default class BufferedFormControl extends React.PureComponent {
  render() {
    return <Buffered {...this.props}>
      <FormControl />
    </Buffered>
  }
}
import { faChevronCircleLeft, faChevronCircleRight } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Button, Container, Col, Row } from "react-bootstrap";

type PaginateProps = {
  children: React.ReactElement[],
  itemsPerPage: number
};

type PaginateState = {
  page: number
};

export default class Paginate extends React.Component<PaginateProps, PaginateState> {
  readonly state: PaginateState = { page: 0 };
  // componentDidUpdate = (prevProps: PaginateProps<T>, prevState: PaginateState): PaginateState => {
  //   if (prevState.page > Math.floor(this.props.items.length / this.props.itemsPerPage))
  //     return Math.floor(this.props.items.length / this.props.itemsPerPage);
  // }

  static getDerivedStateFromProps(props: PaginateProps, state: PaginateState) {
    const n_pages = Math.floor(props.children.length / props.itemsPerPage) + 1;
    return (state.page >= n_pages) ? { page: n_pages - 1 } : null;
  }

  render() {
    const { children, itemsPerPage } = this.props;
    const { page } = this.state;

    const n_pages = Math.floor(children.length / itemsPerPage) + 1;

    return <Container className="paginate">
      <Row className="paginate-body">
        <Col>
          {
            children.slice(page * itemsPerPage, Math.min(children.length, (page + 1)*itemsPerPage))
          }
        </Col>
      </Row>
      <Row className="paginate-footer">
        <Col className="paginate-prev" md={2}>
          <Button variant="link" disabled={page === 0} onClick={() => this.setState({ page: page - 1 })}> <FontAwesomeIcon icon={faChevronCircleLeft} /> </Button>
        </Col>
        <Col className="paginate-page">
          { page + 1 } of { n_pages }
        </Col>
        <Col className="paginate-next" md={2}>
          <Button variant="link" disabled={page === (n_pages-1)} onClick={() => this.setState({ page: page + 1 })}> <FontAwesomeIcon icon={faChevronCircleRight} /> </Button>
        </Col>
      </Row>
    </Container>
  }
}
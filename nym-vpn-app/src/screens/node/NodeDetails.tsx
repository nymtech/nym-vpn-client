import * as H from 'history';
import { useLocation } from 'react-router';
import { UiGateway } from '../../contexts';

type RouteState = {
  gateway: UiGateway;
};

function NodeDetails() {
  const location = useLocation() as H.Location<RouteState>;
  console.log('_-_-_-_-_-_-_-_-_-', location.state);

  return <div></div>;
}

export default NodeDetails;

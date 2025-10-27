const React = require('react');

const SvgMock = React.forwardRef((props, ref) => {
  return React.createElement('div', {
    ...props,
    ref,
    'data-testid': 'mocked-svg',
  });
});

SvgMock.displayName = 'SvgMock';

module.exports = SvgMock;

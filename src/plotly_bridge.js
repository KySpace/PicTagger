function axisRange(axisType, limits, minIndex, maxIndex) {
  if (!limits) {
    return undefined;
  }

  const min = limits[minIndex];
  const max = limits[maxIndex];
  if (axisType === "log") {
    if (min <= 0 || max <= 0) {
      return undefined;
    }
    return [Math.log10(min), Math.log10(max)];
  }
  return [min, max];
}

function markerOpacity(points) {
  if (points.length === 0) {
    return [];
  }

  const weights = points.map((point) => point.weight);
  const min = Math.min(...weights);
  const max = Math.max(...weights);
  if (min === max) {
    return points.map(() => 0.85);
  }

  return weights.map((weight) => 0.25 + ((weight - min) / (max - min)) * 0.75);
}

export function renderPlotlyScatter(
  element,
  payloadJson,
  onSelect,
  onHover,
  onUnhover,
  onMark,
  onMarkPositions,
) {
  if (!globalThis.Plotly) {
    element.innerHTML =
      '<div class="plotly-missing">Plotly failed to load. Check network access for the Plotly script.</div>';
    return;
  }

  const payload = JSON.parse(payloadJson);
  const points = payload.points ?? [];
  const selectedId = payload.selected_id;
  const threshold = Number(payload.weight_threshold ?? 0);
  const baseMarkerSize = 9;
  const customData = points.map((point) => [point.id, point.pair_index]);
  const isBelowThreshold = (point) =>
    payload.threshold_mode && point.weight < threshold;
  const lineWidths = points.map((point) => {
    if (point.id === selectedId) {
      return 2.5;
    }
    return isBelowThreshold(point) ? 1 : 0;
  });
  const lineColors = points.map((point) => {
    if (point.id === selectedId) {
      return "#c66a00";
    }
    return isBelowThreshold(point) ? point.color : "transparent";
  });

  element.__pictaggerOnSelect = onSelect;
  element.__pictaggerOnHover = onHover;
  element.__pictaggerOnUnhover = onUnhover;
  element.__pictaggerOnMark = onMark;
  element.__pictaggerOnMarkPositions = onMarkPositions;
  element.__pictaggerMarkMode = Boolean(payload.mark_mode);
  element.__pictaggerPointCount = points.length;
  element.__pictaggerPoints = points;

  const axisPixel = (axis, value) => {
    if (axis && typeof axis.d2p === "function") {
      return axis.d2p(value);
    }
    if (axis && typeof axis.l2p === "function") {
      return axis.l2p(value);
    }
    return undefined;
  };

  const pointScreenPosition = (point, event) => {
    const rect = element.getBoundingClientRect();
    const xaxis = point?.xaxis ?? element._fullLayout?.xaxis;
    const yaxis = point?.yaxis ?? element._fullLayout?.yaxis;
    const xPixel = axisPixel(xaxis, point?.x);
    const yPixel = axisPixel(yaxis, point?.y);
    const dotX =
      Number.isFinite(xPixel)
        ? rect.left + (xaxis?._offset ?? 0) + xPixel
        : event?.clientX ?? 24;
    const dotY =
      Number.isFinite(yPixel)
        ? rect.top + (yaxis?._offset ?? 0) + yPixel
        : event?.clientY ?? 24;
    return [dotX, dotY];
  };

  const pointLikeFromDataPoint = (point) => ({
    x: point.ib,
    y: point.frequency,
    customdata: [point.id, point.pair_index],
  });

  const nearestPointForEvent = (event, maxDistance = 24) => {
    const xaxis = element._fullLayout?.xaxis;
    const yaxis = element._fullLayout?.yaxis;
    if (!xaxis || !yaxis) {
      return undefined;
    }
    let nearest;
    let nearestDistance = maxDistance;
    for (const point of element.__pictaggerPoints ?? []) {
      const [dotX, dotY] = pointScreenPosition(pointLikeFromDataPoint(point), event);
      if (!Number.isFinite(dotX) || !Number.isFinite(dotY)) {
        continue;
      }
      const distance = Math.hypot(
        dotX - (event?.clientX ?? 0),
        dotY - (event?.clientY ?? 0),
      );
      if (distance <= nearestDistance) {
        nearestDistance = distance;
        nearest = point;
      }
    }
    return nearest ? pointLikeFromDataPoint(nearest) : undefined;
  };

  const markPoint = (point, event) => {
    const id = point?.customdata?.[0];
    const pairIndex = point?.customdata?.[1] ?? 0;
    if (!id) {
      return;
    }
    const clientX = event?.clientX ?? 24;
    const clientY = event?.clientY ?? 24;
    const [dotX, dotY] = pointScreenPosition(point, event);
    element.__pictaggerOnMark(
      `${id}:${pairIndex}:${dotX}:${dotY}:${clientX}:${clientY}`,
    );
  };

  const clearHoverPointState = () => {
    element.__pictaggerLastHoverPoint = undefined;
    element.__pictaggerLastHoverAt = undefined;
    element.__pictaggerLastHoverScreenPosition = undefined;
  };

  const updateMarkedPointPositions = () => {
    if (!element.__pictaggerMarkMode || !element.__pictaggerOnMarkPositions) {
      return;
    }
    const rect = element.getBoundingClientRect();
    const xaxis = element._fullLayout?.xaxis;
    const yaxis = element._fullLayout?.yaxis;
    if (!xaxis || !yaxis) {
      return;
    }
    const positions = (element.__pictaggerPoints ?? [])
      .map((point) => {
        const xPixel = axisPixel(xaxis, point.ib);
        const yPixel = axisPixel(yaxis, point.frequency);
        if (!Number.isFinite(xPixel) || !Number.isFinite(yPixel)) {
          return undefined;
        }
        return {
          id: point.id,
          pair_index: point.pair_index,
          dot_x: rect.left + (xaxis._offset ?? 0) + xPixel,
          dot_y: rect.top + (yaxis._offset ?? 0) + yPixel,
        };
      })
      .filter(Boolean);
    if (positions.length > 0) {
      element.__pictaggerOnMarkPositions(JSON.stringify(positions));
    }
  };

  const scheduleMarkedPointPositionUpdate = () => {
    if (!element.__pictaggerMarkMode) {
      return;
    }
    if (element.__pictaggerMarkPositionFrame) {
      cancelAnimationFrame(element.__pictaggerMarkPositionFrame);
    }
    element.__pictaggerMarkPositionFrame = requestAnimationFrame(() => {
      element.__pictaggerMarkPositionFrame = undefined;
      updateMarkedPointPositions();
    });
  };
  const scheduleSettledMarkedPointPositionUpdate = () => {
    element.__pictaggerScheduleMarkedPointPositionUpdate?.();
    setTimeout(() => {
      element.__pictaggerScheduleMarkedPointPositionUpdate?.();
    }, 40);
    setTimeout(() => {
      element.__pictaggerScheduleMarkedPointPositionUpdate?.();
    }, 120);
  };
  element.__pictaggerScheduleMarkedPointPositionUpdate =
    scheduleMarkedPointPositionUpdate;
  element.__pictaggerScheduleSettledMarkedPointPositionUpdate =
    scheduleSettledMarkedPointPositionUpdate;

  const trace = {
    type: "scatter",
    mode: "markers",
    x: points.map((point) => point.ib),
    y: points.map((point) => point.frequency),
    customdata: customData,
    hoverinfo: "none",
    marker: {
      size: points.map(() => baseMarkerSize),
      color: points.map((point) =>
        isBelowThreshold(point) ? "rgba(255,255,255,0)" : point.color,
      ),
      opacity: payload.threshold_mode ? 1 : markerOpacity(points),
      line: {
        color: lineColors,
        width: lineWidths,
      },
    },
  };

  const layout = {
    autosize: true,
    uirevision: `scatter-axis-${payload.axis_view_revision ?? 0}`,
    margin: { l: 58, r: 18, t: 12, b: 48 },
    dragmode: "pan",
    hovermode: "closest",
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "#f9fcf8",
    xaxis: {
      title: { text: "IB" },
      type: payload.x_axis_type,
      range: axisRange(payload.x_axis_type, payload.manual_limits, 0, 1),
      fixedrange: Boolean(payload.x_limits_fixed),
      zeroline: false,
      gridcolor: "#d6ddd2",
      linecolor: "#68726f",
      mirror: true,
    },
    yaxis: {
      title: { text: "frequency" },
      type: payload.y_axis_type,
      range: axisRange(payload.y_axis_type, payload.manual_limits, 2, 3),
      fixedrange: Boolean(payload.y_limits_fixed),
      zeroline: false,
      gridcolor: "#d6ddd2",
      linecolor: "#68726f",
      mirror: true,
    },
    showlegend: false,
  };

  const config = {
    responsive: true,
    scrollZoom: true,
    displaylogo: false,
    modeBarButtonsToRemove: ["lasso2d", "select2d"],
  };

  globalThis.Plotly.react(element, [trace], layout, config).then(() => {
    element.__pictaggerScheduleSettledMarkedPointPositionUpdate?.();

    if (element.__pictaggerPlotlyHandlers || typeof element.on !== "function") {
      return;
    }

    element.on("plotly_click", (event) => {
      const nativeEvent = event?.event;
      if (element.__pictaggerMarkMode && nativeEvent?.button === 2) {
        nativeEvent.preventDefault();
        markPoint(event?.points?.[0], nativeEvent);
        return;
      }
      const id = event?.points?.[0]?.customdata?.[0];
      if (id) {
        element.__pictaggerOnSelect(id);
      }
    });
    element.on("plotly_hover", (event) => {
      const id = event?.points?.[0]?.customdata?.[0];
      const pairIndex = event?.points?.[0]?.customdata?.[1];
      const pointNumber = event?.points?.[0]?.pointNumber;
      const clientX = event?.event?.clientX ?? 24;
      const clientY = event?.event?.clientY ?? 24;
      const point = event?.points?.[0];
      element.__pictaggerLastHoverPoint = point;
      element.__pictaggerLastHoverAt = Date.now();
      element.__pictaggerLastHoverScreenPosition = pointScreenPosition(
        point,
        event?.event,
      );
      if (element.__pictaggerHoverClearTimer) {
        clearTimeout(element.__pictaggerHoverClearTimer);
        element.__pictaggerHoverClearTimer = undefined;
      }
      if (
        Number.isInteger(pointNumber) &&
        element.__pictaggerHoveredPointNumber !== pointNumber
      ) {
        element.__pictaggerHoveredPointNumber = pointNumber;
        const sizes = Array.from(
          { length: element.__pictaggerPointCount ?? 0 },
          (_, index) => (index === pointNumber ? baseMarkerSize * 1.2 : baseMarkerSize),
        );
        globalThis.Plotly.restyle(element, { "marker.size": [sizes] }, [0]);
      }
      if (id) {
        element.__pictaggerOnHover(
          `${id}:${pairIndex ?? 0}:${clientX}:${clientY}`,
        );
      }
    });
    element.on("plotly_unhover", () => {
      if (element.__pictaggerHoverClearTimer) {
        clearTimeout(element.__pictaggerHoverClearTimer);
      }
      element.__pictaggerHoverClearTimer = setTimeout(() => {
        element.__pictaggerHoverClearTimer = undefined;
        element.__pictaggerHoveredPointNumber = undefined;
        clearHoverPointState();
        const sizes = Array.from(
          { length: element.__pictaggerPointCount ?? 0 },
          () => baseMarkerSize,
        );
        globalThis.Plotly.restyle(element, { "marker.size": [sizes] }, [0]);
        element.__pictaggerOnUnhover();
      }, 90);
    });
    element.on("plotly_relayout", () => {
      clearHoverPointState();
      element.__pictaggerScheduleSettledMarkedPointPositionUpdate?.();
    });
    element.on("plotly_relayouting", () => {
      element.__pictaggerScheduleMarkedPointPositionUpdate?.();
    });
    element.on("plotly_afterplot", () => {
      element.__pictaggerScheduleSettledMarkedPointPositionUpdate?.();
    });
    element.addEventListener("contextmenu", (event) => {
      if (!element.__pictaggerMarkMode) {
        return;
      }
      const point = nearestPointForEvent(event, 26);
      if (point) {
        event.preventDefault();
        markPoint(point, event);
      }
    });
    element.addEventListener(
      "pointermove",
      (event) => {
        if (!element.__pictaggerMarkMode) {
          return;
        }
        const point = nearestPointForEvent(event, 18);
        if (!point) {
          return;
        }
        const id = point.customdata?.[0];
        const pairIndex = point.customdata?.[1] ?? 0;
        if (!id) {
          return;
        }
        element.__pictaggerLastHoverPoint = point;
        element.__pictaggerLastHoverAt = Date.now();
        element.__pictaggerLastHoverScreenPosition = pointScreenPosition(
          point,
          event,
        );
        element.__pictaggerOnHover(
          `${id}:${pairIndex}:${event.clientX ?? 24}:${event.clientY ?? 24}`,
        );
      },
      { passive: true },
    );
    element.addEventListener("pointerleave", () => {
      clearHoverPointState();
    });
    if (globalThis.ResizeObserver) {
      element.__pictaggerResizeObserver = new ResizeObserver(() => {
        element.__pictaggerScheduleSettledMarkedPointPositionUpdate?.();
      });
      element.__pictaggerResizeObserver.observe(element);
    }
    globalThis.addEventListener(
      "scroll",
      () => {
        element.__pictaggerScheduleSettledMarkedPointPositionUpdate?.();
      },
      { passive: true },
    );
    globalThis.addEventListener("resize", () => {
      element.__pictaggerScheduleSettledMarkedPointPositionUpdate?.();
    });
    globalThis.visualViewport?.addEventListener(
      "scroll",
      () => {
        element.__pictaggerScheduleSettledMarkedPointPositionUpdate?.();
      },
      { passive: true },
    );
    globalThis.visualViewport?.addEventListener("resize", () => {
      element.__pictaggerScheduleSettledMarkedPointPositionUpdate?.();
    });
    element.__pictaggerPlotlyHandlers = true;
  });
}

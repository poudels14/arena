import matplotlib
from matplotlib.backends import backend_svg, backend_agg
from matplotlib import pyplot as plt
from matplotlib.backend_bases import _Backend
from random import randrange
import portal_core

@_Backend.export
class _BackendPortal(_Backend):
    FigureCanvas = backend_agg.FigureCanvas
    FigureManager = backend_agg.FigureManager

    @staticmethod
    def show(*args, **kwargs):
        plt.savefig(portal_core.get_artifacts_path() + "/plot-" + str(randrange(1000)) + ".png")


matplotlib.use("module://portal.matplotlib")
